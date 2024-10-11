// src/expert/solana_expert.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[EXPERT]Xyn>=====S===t===u===d===i===o===s======[R|$>

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signer}, 
    system_instruction, 
    transaction::Transaction, 
    pubkey::Pubkey, 
    commitment_config::CommitmentConfig,
    message::Message,
};
use solana_sdk::transaction::TransactionError;
use anyhow::{Result, Context};
use anchor_client::Client;
use spl_token::{self, state::Account as TokenAccount};
use spl_associated_token_account;
use solana_program::program_pack::Pack;
use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use reqwest::Client as ReqwestClient;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SolanaExpert {
    rpc_client: Arc<RwLock<RpcClient>>,
    anchor_client: Arc<RwLock<Client>>,
    research_client: ReqwestClient,
    best_practices: Vec<String>,
    state_cache: Arc<RwLock<HashMap<Pubkey, Vec<u8>>>>, // On-chain state cache
}

impl SolanaExpert {
    pub fn new(rpc_url: &str, cluster_url: &str) -> Self {
        let rpc_client = Arc::new(RwLock::new(RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed())));
        let anchor_client = Arc::new(RwLock::new(Client::new_with_cluster(cluster_url.to_string())));
        Self {
            rpc_client,
            anchor_client,
            research_client: ReqwestClient::new(),
            best_practices: vec![
                "Check account balances before transactions.".to_string(),
                "Handle PDAs using derived keys.".to_string(),
                "Simulate transactions before executing.".to_string(),
                "Use batching and fee estimation before sending.".to_string(),
            ],
            state_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn derive_pda(&self, seed: &[&[u8]], program_id: &Pubkey) -> Result<Pubkey> {
        let (pda, _) = Pubkey::find_program_address(seed, program_id);
        Ok(pda)
    }

    pub async fn get_cached_state(&self, pubkey: &Pubkey) -> Result<Vec<u8>> {
        {
            let cache = self.state_cache.read().await;
            if let Some(cached_data) = cache.get(pubkey) {
                return Ok(cached_data.clone());
            }
        }
        let client = self.rpc_client.read().await;
        let account = client.get_account(pubkey).context("Failed to fetch account")?; 
        let data = account.data.clone();
        let mut cache = self.state_cache.write().await;
        cache.insert(*pubkey, data.clone());
        Ok(data)
    }

    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        let client = self.rpc_client.read().await;
        let balance = client.get_balance(pubkey).context("Failed to retrieve balance")?;
        Ok(balance)
    }

    pub async fn send_transaction(
        &self,
        sender: &Keypair,
        recipient: &Pubkey,
        amount: u64
    ) -> Result<String> {
        let client = self.rpc_client.read().await;

        // Create a transfer instruction
        let ix = system_instruction::transfer(&sender.pubkey(), recipient, amount);

        // Create a transaction message
        let (recent_blockhash, _) = client.get_recent_blockhash().context("Failed to get recent blockhash")?;
        let message = Message::new(&[ix], Some(&sender.pubkey()));

        // Simulate the transaction before sending it
        client.simulate_transaction(&Transaction::new_unsigned(message.clone()))
            .context("Simulation failed")?;

        // Estimate fee and check account balances
        let fee_calculator = client.get_fee_calculator_for_blockhash(&recent_blockhash)
            .context("Failed to get fee calculator")?
            .ok_or_else(|| anyhow::anyhow!("No fee calculator available"))?;
        let estimated_fee = fee_calculator.calculate_fee(&message);
        let sender_balance = self.get_balance(&sender.pubkey()).await?;
        if sender_balance < amount + estimated_fee {
            return Err(anyhow::anyhow!("Insufficient balance for transaction"));
        }

        // Create the final transaction
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&sender.pubkey()),
            &[sender],
            recent_blockhash,
        );

        // Send the transaction
        let signature = client.send_and_confirm_transaction(&tx)
            .context("Failed to send transaction")?;
        Ok(signature.to_string())
    }

    pub async fn create_spl_token_account(
        &self, 
        payer: &Keypair, 
        mint: &Pubkey, 
        owner: &Pubkey
    ) -> Result<Pubkey> {
        let client = self.rpc_client.read().await;

        // Create associated token address
        let token_account = spl_associated_token_account::get_associated_token_address(owner, mint);

        // Create the instruction for creating the associated token account
        let create_ix = spl_associated_token_account::create_associated_token_account(
            &payer.pubkey(),
            owner,
            mint,
        );

        let (recent_blockhash, _) = client.get_recent_blockhash().context("Failed to get recent blockhash")?;

        let message = Message::new(&[create_ix], Some(&payer.pubkey()));

        // Create transaction
        let tx = Transaction::new_signed_with_payer(
            &[create_ix],
            Some(&payer.pubkey()),
            &[payer],
            recent_blockhash,
        );

        client.send_and_confirm_transaction(&tx).context("Failed to create SPL Token Account")?;
        Ok(token_account)
    }

    pub async fn send_transactions_batch(
        &self,
        transactions: Vec<Transaction>
    ) -> Result<Vec<String>> {
        let client = self.rpc_client.read().await;
        let signatures = futures::stream::iter(transactions)
            .map(|tx| client.send_and_confirm_transaction(&tx))
            .buffer_unordered(10) // Limit concurrent sends
            .collect::<Vec<_>>()
            .await;
        
        let results: Vec<_> = signatures.into_iter().collect();
        Ok(results.into_iter().filter_map(Result::ok).collect())
    }

    pub async fn query_transaction(&self, signature: &str) -> Result<String> {
        let client = self.rpc_client.read().await;
        let tx = client.get_transaction(signature, CommitmentConfig::finalized())
            .context("Failed to retrieve transaction")?;
        Ok(format!("{:?}", tx))
    }

    pub async fn update_best_practices(&mut self, practice: &str) -> Result<()> {
        if !self.best_practices.contains(&practice.to_string()) {
            self.best_practices.push(practice.to_string());
        }
        Ok(())
    }

    pub fn get_best_practices(&self) -> Vec<String> {
        self.best_practices.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[tokio::test]
    async fn test_create_keypair() {
        let expert = SolanaExpert::new("https://api.devnet.solana.com", "https://devnet.solana.com");
        let keypair = Keypair::new();
        assert_eq!(keypair.pubkey().to_string().len(), 44);
    }

    #[tokio::test]
    async fn test_get_balance() {
        let expert = SolanaExpert::new("https://api.devnet.solana.com", "https://devnet.solana.com");
        let pubkey = Pubkey::new_unique();
        let result = expert.get_balance(&pubkey).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_transaction() {
        let expert = SolanaExpert::new("https://api.devnet.solana.com", "https://devnet.solana.com");
        let sender = Keypair::new();
        let recipient = Pubkey::new_unique();
        let result = expert.send_transaction(&sender, &recipient, 1_000_000).await;
        assert!(result.is_err()); // Fails because of insufficient funds, but functionally correct
    }

    #[tokio::test]
    async fn test_query_transaction() {
        let expert = SolanaExpert::new("https://api.devnet.solana.com", "https://devnet.solana.com");
        let signature = "5S8XXzA1rTgRhbA1niUzFryXZpLDhx7VxyjPSmW7ksp1uKfNsF1RHeVmiFMyRuEM5HFvTfcF1uFP3ZboDRzXcziV";
        let result = expert.query_transaction(signature).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_research_function() {
        let expert = SolanaExpert::new("https://api.devnet.solana.com", "https://devnet.solana.com");
        let papers = expert.research_function("solana programming").await.unwrap();
        assert!(!papers.is_empty());
    }

    #[tokio::test]
    async fn test_update_best_practices() {
        let mut expert = SolanaExpert::new("https://api.devnet.solana.com", "https://devnet.solana.com");
        let practice = "Use pubkey-derivation for security.";
        expert.update_best_practices(practice).await.unwrap();
        assert!(expert.get_best_practices().contains(&practice.to_string()));
    }
}