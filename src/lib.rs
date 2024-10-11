// src/lib.rs ~=#######D]======A===r===c====M===o===o===n=====<Lord[LIB]Xyn>=====S===t===u===d===i===o===s======[R|$>
// src/lib.rs

// Core modules
pub mod aproar;
pub mod constants;
pub mod data_processing;
pub mod expert;
pub mod lsm;
pub mod model_interpretability;
pub mod machines;
pub mod models;
pub mod multi_modal;
pub mod omnixelerator;
pub mod omnixtracker;
pub mod security;
pub mod transformers;
pub mod utils;

// Re-exports for convenient access
pub use aproar::{
    compression::{
        LZ4Compression,
        ZstdCompression,
        CompressionStrategy,
        CompressionManager,
    },
    memory::{
        AddressingMechanism,
        NTMController,
        NTMMemory,
        ReadHead,
    },    
    ntm::{
        AddressingMechanism,
        NTMController,
        NTMMemory,
        ReadHead,
    },
    storage::{
        HDF5Storage,
        ParquetStorage,
        TileDBStorage,
    },
    retrieval::{
        RedisCache,
        RocksDBPersistence,
    },
    AproarManager,
};

pub use omnixtracker::{
    OmniXError,
    OmniXMetry,
};

pub use constants::*;

pub use data_processing::{
    Dataset,
    Tokenizer,
};

pub use expert::{
    BevyExpert,
    ResearchExpert,
    RustExpert,
    SolanaExpert,
};

pub use lsm::{
    Neuron,
    Readout,
    Reservoir,
    Synapse,
    LSMEncoder,
};

pub use model_interpretability::{
    DecisionVisualization,
    FeatureImportance,
    LIMEInterpreter,
    SHAPInterpreter,
};

pub use machines::{
    LiquidStateMachine,
    NeuralTuringMachine,
};

pub use models::{
    GPTNeoModel,
    RustBERTModel,
};

pub use multi_modal::{
    AudioProcessor,
    DataFusion,
    ImageProcessor,
    TextProcessor,
};

pub use security::{
    AESEncryption,
    WebAuthnAuthentication,
};

pub use transformers::{
    Attention,
    TransformerDecoder,
    TransformerEncoder,
    FeedForward,
    LayerNorm,
};

pub use utils::{
    DataPipeline,
    LordXynSignatureLine,
};