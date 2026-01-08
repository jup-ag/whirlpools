use solana_program_error::{ProgramError, ToStr};

#[derive(Debug)]
pub enum OrcaError {
    InvalidEnum = 6000,                      // 0x1770 (6000)
    InvalidStartTick,                        // 0x1771 (6001)
    TickArrayExistInPool,                    // 0x1772 (6002)
    TickArrayIndexOutofBounds,               // 0x1773 (6003)
    InvalidTickSpacing,                      // 0x1774 (6004)
    ClosePositionNotEmpty,                   // 0x1775 (6005)
    DivideByZero,                            // 0x1776 (6006)
    NumberCastError,                         //  0x1777 (6007)
    NumberDownCastError,                     //  0x1778 (6008)
    TickNotFound,                            // 0x1779 (6009)
    InvalidTickIndex,                        // 0x177a (6010)
    SqrtPriceOutOfBounds,                    // 0x177b (6011)
    LiquidityZero,                           // 0x177c (6012)
    LiquidityTooHigh,                        // 0x177d (6013)
    LiquidityOverflow,                       // 0x177e (6014)
    LiquidityUnderflow,                      // 0x177f (6015)
    LiquidityNetError,                       // 0x1780 (6016)
    TokenMaxExceeded,                        // 0x1781 (6017)
    TokenMinSubceeded,                       // 0x1782 (6018)
    MissingOrInvalidDelegate,                // 0x1783 (6019)
    InvalidPositionTokenAmount,              // 0x1784 (6020)
    InvalidTimestampConversion,              // 0x1785 (6021)
    InvalidTimestamp,                        // 0x1786 (6022)
    InvalidTickArraySequence,                // 0x1787 (6023)
    InvalidTokenMintOrder,                   // 0x1788 (6024)
    RewardNotInitialized,                    // 0x1789 (6025)
    InvalidRewardIndex,                      // 0x178a (6026)
    RewardVaultAmountInsufficient,           // 0x178b (6027)
    FeeRateMaxExceeded,                      // 0x178c (6028)
    ProtocolFeeRateMaxExceeded,              // 0x178d (6029)
    MultiplicationShiftRightOverflow,        // 0x178e (6030)
    MulDivOverflow,                          // 0x178f (6031)
    MulDivInvalidInput,                      // 0x1790 (6032)
    MultiplicationOverflow,                  // 0x1791 (6033)
    InvalidSqrtPriceLimitDirection,          // 0x1792 (6034)
    ZeroTradableAmount,                      // 0x1793 (6035)
    AmountOutBelowMinimum,                   // 0x1794 (6036)
    AmountInAboveMaximum,                    // 0x1795 (6037)
    TickArraySequenceInvalidIndex,           // 0x1796 (6038)
    AmountCalcOverflow,                      // 0x1797 (6039)
    AmountRemainingOverflow,                 // 0x1798 (6040)
    InvalidIntermediaryMint,                 // 0x1799 (6041)
    DuplicateTwoHopPool,                     // 0x179a (6042)
    InvalidBundleIndex,                      // 0x179b (6043)
    BundledPositionAlreadyOpened,            // 0x179c (6044)
    BundledPositionAlreadyClosed,            // 0x179d (6045)
    PositionBundleNotDeletable,              // 0x179e (6046)
    UnsupportedTokenMint,                    // 0x179f (6047)
    RemainingAccountsInvalidSlice,           // 0x17a0 (6048)
    RemainingAccountsInsufficient,           // 0x17a1 (6049)
    NoExtraAccountsForTransferHook,          // 0x17a2 (6050)
    IntermediateTokenAmountMismatch,         // 0x17a3 (6051)
    TransferFeeCalculationError,             // 0x17a4 (6052)
    RemainingAccountsDuplicatedAccountsType, // 0x17a5 (6053)
    FullRangeOnlyPool,                       // 0x17a6 (6054)
    TooManySupplementalTickArrays,           // 0x17a7 (6055)
    DifferentWhirlpoolTickArrayAccount,      // 0x17a8 (6056)
    PartialFillError,                        // 0x17a9 (6057)
    PositionNotLockable,                     // 0x17aa (6058)
    OperationNotAllowedOnLockedPosition,     // 0x17ab (6059)
    SameTickRangeNotAllowed,                 // 0x17ac (6060)
    InvalidAdaptiveFeeConstants,             // 0x17ad (6061)
    InvalidFeeTierIndex,                     // 0x17ae (6062)
    InvalidTradeEnableTimestamp,             // 0x17af (6063)
    TradeIsNotEnabled,                       // 0x17b0 (6064)
    RentCalculationError,                    // 0x17b1 (6065)
}

impl ToStr for OrcaError {
    fn to_str(&self) -> &'static str {
        match self {
            OrcaError::InvalidEnum => "Enum value could not be converted",
            OrcaError::InvalidStartTick => "Invalid start tick index provided.",
            OrcaError::TickArrayExistInPool => "Tick-array already exists in this whirlpool",
            OrcaError::TickArrayIndexOutofBounds => "Attempt to search for a tick-array failed",
            OrcaError::InvalidTickSpacing => "Tick-spacing is not supported",
            OrcaError::ClosePositionNotEmpty => "Position is not empty It cannot be closed",
            OrcaError::DivideByZero => "Unable to divide by zero",
            OrcaError::NumberCastError => "Unable to cast number into BigInt",
            OrcaError::NumberDownCastError => "Unable to down cast number",
            OrcaError::TickNotFound => "Tick not found within tick array",
            OrcaError::InvalidTickIndex => {
                "Provided tick index is either out of bounds or uninitializable"
            }
            OrcaError::SqrtPriceOutOfBounds => "Provided sqrt price out of bounds",
            OrcaError::LiquidityZero => "Liquidity amount must be greater than zero",
            OrcaError::LiquidityTooHigh => "Liquidity amount must be less than i64::MAX",
            OrcaError::LiquidityOverflow => "Liquidity overflow",
            OrcaError::LiquidityUnderflow => "Liquidity underflow",
            OrcaError::LiquidityNetError => "Tick liquidity net underflowed or overflowed",
            OrcaError::TokenMaxExceeded => "Exceeded token max",
            OrcaError::TokenMinSubceeded => "Did not meet token min",
            OrcaError::MissingOrInvalidDelegate => {
                "Position token account has a missing or invalid delegate"
            }
            OrcaError::InvalidPositionTokenAmount => "Position token amount must be 1",
            OrcaError::InvalidTimestampConversion => {
                "Timestamp should be convertible from i64 to u64"
            }
            OrcaError::InvalidTimestamp => {
                "Timestamp should be greater than the last updated timestamp"
            }
            OrcaError::InvalidTickArraySequence => {
                "Invalid tick array sequence provided for instruction."
            }
            OrcaError::InvalidTokenMintOrder => "Token Mint in wrong order",
            OrcaError::RewardNotInitialized => "Reward not initialized",
            OrcaError::InvalidRewardIndex => "Invalid reward index",
            OrcaError::RewardVaultAmountInsufficient => {
                "Reward vault requires amount to support emissions for at least one day"
            }
            OrcaError::FeeRateMaxExceeded => "Exceeded max fee rate",
            OrcaError::ProtocolFeeRateMaxExceeded => "Exceeded max protocol fee rate",
            OrcaError::MultiplicationShiftRightOverflow => {
                "Multiplication with shift right overflow"
            }
            OrcaError::MulDivOverflow => "Muldiv overflow",
            OrcaError::MulDivInvalidInput => "Invalid div_u256 input",
            OrcaError::MultiplicationOverflow => "Multiplication overflow",
            OrcaError::InvalidSqrtPriceLimitDirection => {
                "Provided SqrtPriceLimit not in the same direction as the swap."
            }
            OrcaError::ZeroTradableAmount => "There are no tradable amount to swap.",
            OrcaError::AmountOutBelowMinimum => "Amount out below minimum threshold",
            OrcaError::AmountInAboveMaximum => "Amount in above maximum threshold",
            OrcaError::TickArraySequenceInvalidIndex => "Invalid index for tick array sequence",
            OrcaError::AmountCalcOverflow => "Amount calculated overflows",
            OrcaError::AmountRemainingOverflow => "Amount remaining overflows",
            OrcaError::InvalidIntermediaryMint => "Invalid intermediary mint",
            OrcaError::DuplicateTwoHopPool => "Duplicate two hop pool",
            OrcaError::InvalidBundleIndex => "Bundle index is out of bounds",
            OrcaError::BundledPositionAlreadyOpened => "Position has already been opened",
            OrcaError::BundledPositionAlreadyClosed => "Position has already been closed",
            OrcaError::PositionBundleNotDeletable => {
                "Unable to delete PositionBundle with open positions"
            }
            OrcaError::UnsupportedTokenMint => "Token mint has unsupported attributes",
            OrcaError::RemainingAccountsInvalidSlice => "Invalid remaining accounts",
            OrcaError::RemainingAccountsInsufficient => "Insufficient remaining accounts",
            OrcaError::NoExtraAccountsForTransferHook => {
                "Unable to call transfer hook without extra accounts"
            }
            OrcaError::IntermediateTokenAmountMismatch => "Output and input amount mismatch",
            OrcaError::TransferFeeCalculationError => "Transfer fee calculation failed",
            OrcaError::RemainingAccountsDuplicatedAccountsType => {
                "Same accounts type is provided more than once"
            }
            OrcaError::FullRangeOnlyPool => "This whirlpool only supports full-range positions",
            OrcaError::TooManySupplementalTickArrays => {
                "Too many supplemental tick arrays provided"
            }
            OrcaError::DifferentWhirlpoolTickArrayAccount => {
                "TickArray account for different whirlpool provided"
            }
            OrcaError::PartialFillError => "Trade resulted in partial fill",
            OrcaError::PositionNotLockable => "Position is not lockable",
            OrcaError::OperationNotAllowedOnLockedPosition => {
                "Operation not allowed on locked position"
            }
            OrcaError::SameTickRangeNotAllowed => {
                "Cannot reset position range with same tick range"
            }
            OrcaError::InvalidAdaptiveFeeConstants => "Invalid adaptive fee constants",
            OrcaError::InvalidFeeTierIndex => "Invalid fee tier index",
            OrcaError::InvalidTradeEnableTimestamp => "Invalid trade enable timestamp",
            OrcaError::TradeIsNotEnabled => "Trade is not enabled yet",
            OrcaError::RentCalculationError => "Rent calculation error",
        }
    }
}

impl From<OrcaError> for ProgramError {
    fn from(value: OrcaError) -> Self {
        ProgramError::Custom(value as u32)
    }
}
