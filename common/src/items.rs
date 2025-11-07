use crate::random::ItemChoice;
use strum::EnumIter;
#[cfg(feature = "std")]
use core::fmt::{self, Display};

pub enum RandomSource {
    Shop,
    Soul,
    BuffonPack,
    Wraith,
    RareTag,
    UncommonTag,
    Arcana,
    Celestial,
}

#[derive(Debug, EnumIter, Copy, Clone)]
pub enum Tags {
    UncommonTag,
    RareTag,
    NegativeTag,
    FoilTag,
    HolographicTag,
    PolychromeTag,
    InvestmentTag,
    VoucherTag,
    BossTag,
    StandardTag,
    CharmTag,
    MeteorTag,
    BuffoonTag,
    HandyTag,
    GarbageTag,
    EtherealTag,
    CouponTag,
    DoubleTag,
    JuggleTag,
    D6Tag,
    TopUpTag,
    SpeedTag,
    OrbitalTag,
    EconomyTag,
}

impl ItemChoice for Tags {
    fn retry(&self) -> bool { false }
    fn locked(&self) -> bool { false }
}

#[cfg(feature = "std")]
impl Display for RandomSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RandomSource::Shop => write!(f, "sho"),
            RandomSource::BuffonPack => write!(f, "buf"),
            RandomSource::Wraith => write!(f, "wra"),
            RandomSource::RareTag => write!(f, "rta"),
            RandomSource::UncommonTag => write!(f, "uta"),
            RandomSource::Soul => write!(f, "sou"),
            RandomSource::Arcana => write!(f, "ar1"),
            RandomSource::Celestial => write!(f, "pl1"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Editions {
    Negative,
    Polychrome,
    Holographic,
    Foil,
    None,
}

#[derive(Debug)]
pub enum JokerRarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

#[derive(Debug, EnumIter)]
pub enum JokerStickers {
    Eternal,
    Perishable,
    Rental,
}

#[derive(Debug)]
pub struct Joker {
    pub joker: JokerTypes,
    pub rarity: JokerRarity,
    pub edition: Editions,
    pub stickers: [bool; 3],
}

#[cfg(feature = "std")]
impl Display for Joker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.stickers
                .iter()
                .zip(JokerStickers::iter())
                .filter_map(|(added, sticker)| if *added {
                    Some(format!("{sticker:?}"))
                } else {
                    None
                })
                .collect::<Vec<String>>()
                .join(" ")
        )?;

        if self.edition != Editions::None {
            write!(f, " ({:?})", self.edition)?;
        }

        write!(f, "{:?}", self.joker)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Pack {
    Buffoon,
    Arcana,
    Spectral,
    Planet
}

impl ItemChoice for Pack {
    fn retry(&self) -> bool { false }
    fn locked(&self) -> bool { false }
}

#[derive(Debug, EnumIter, Copy, Clone)]
pub enum CardTypes {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, EnumIter, Copy, Clone)]
pub enum CardSuits {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SealTypes {
    None,
    Red,
    Blue,
    Gold,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EnhancementTypes {
    None,
    Bonus,
    Mult,
    Wild,
    Glass,
    Steel,
    Stone,
    Gold,
    Lucky,
}

#[derive(Debug)]
pub struct Card {
    pub rank: CardTypes,
    pub suit: CardSuits,
    pub enhancement: EnhancementTypes,
    pub edition: Editions,
    pub seal: SealTypes,
    pub sort_id: usize,
}

#[cfg(feature = "std")]
impl Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.edition != Editions::None {
            write!(f, "{:?} ", self.edition)?;
        }
        if self.seal != SealTypes::None {
            write!(f, "{:?} Seal ", self.seal)?;
        }
        if self.enhancement != EnhancementTypes::None {
            write!(f, "{:?} ", self.enhancement)?;
        }
        write!(f, "{:?} of {:?}", self.rank, self.suit)
    }
}

#[derive(Debug, EnumIter, Copy, Clone)]
pub enum Tarots {
    TheFool,
    TheMagician,
    TheHighPriestess,
    TheEmpress,
    TheEmperor,
    TheHierophant,
    TheLovers,
    TheChariot,
    Justice,
    TheHermit,
    TheWheelOfFortune,
    Strength,
    TheHangedMan,
    Death,
    Temperance,
    TheDevil,
    TheTower,
    TheStar,
    TheMoon,
    TheSun,
    Judgement,
    TheWorld,
}

impl ItemChoice for Tarots {
    fn retry(&self) -> bool {
        false
    }

    fn locked(&self) -> bool {
        false
    }
}

#[derive(Debug, EnumIter, Copy, Clone)]
pub enum Planets {
    Mercury,
    Venus,
    Earth,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
    PlanetX,
    Ceres,
    Eris,
}

impl ItemChoice for Planets {
    fn retry(&self) -> bool {
        false
    }

    fn locked(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub enum Spectral {}

#[derive(Debug, EnumIter, Copy, Clone)]
pub enum Vouchers {
    Overstock,
    OverstockPlus,
    ClearanceSale,
    Liquidation,
    Hone,
    GlowUp,
    RerollSurplus,
    RerollGlut,
    CrystalBall,
    OmenGlobe,
    Telescope,
    Observatory,
    Grabber,
    NachoTong,
    Wasteful,
    Recyclomancy,
    TarotMerchant,
    TarotTycoon,
    PlanetMerchant,
    PlanetTycoon,
    SeedMoney,
    MoneyTree,
    Blank,
    Antimatter,
    MagicTrick,
    Illusion,
    Hieroglyph,
    Petroglyph,
    DirectorsCut,
    Retcon,
    PaintBrush,
    Palette,
}

impl ItemChoice for Vouchers {
    fn retry(&self) -> bool {
        false
    }

    fn locked(&self) -> bool {
        match self {
            Vouchers::OverstockPlus
            | Vouchers::Liquidation
            | Vouchers::RerollGlut
            | Vouchers::OmenGlobe
            | Vouchers::Observatory
            | Vouchers::NachoTong
            | Vouchers::Recyclomancy
            | Vouchers::TarotTycoon
            | Vouchers::PlanetTycoon
            | Vouchers::MoneyTree
            | Vouchers::Antimatter
            | Vouchers::Illusion
            | Vouchers::Petroglyph
            | Vouchers::Retcon
            | Vouchers::Palette => true,
            _ => false,
        }
    }
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq, Eq)]
pub enum Bosses {
    TheArm,
    TheClub,
    TheEye,
    AmberAcorn,
    CeruleanBell,
    CrimsonHeart,
    VerdantLeaf,
    VioletVessel,
    TheFish,
    TheFlint,
    TheGoad,
    TheHead,
    TheHook,
    TheHouse,
    TheManacle,
    TheMark,
    TheMouth,
    TheNeedle,
    TheOx,
    ThePillar,
    ThePlant,
    ThePsychic,
    TheSerpent,
    TheTooth,
    TheWall,
    TheWater,
    TheWheel,
    TheWindow,
}

impl ItemChoice for Bosses {
    fn retry(&self) -> bool {
        false
    }

    fn locked(&self) -> bool {
        match self {
            Bosses::AmberAcorn
            | Bosses::CeruleanBell
            | Bosses::CrimsonHeart
            | Bosses::VerdantLeaf
            | Bosses::VioletVessel => true,
            _ => false,
        }
    }
}

#[derive(Debug, EnumIter, Copy, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum JokerTypes {
    // Common jokers
    Joker,
    GreedyJoker,
    LustyJoker,
    WrathfulJoker,
    GluttonousJoker,
    JollyJoker,
    ZanyJoker,
    MadJoker,
    CrazyJoker,
    DrollJoker,
    SlyJoker,
    WilyJoker,
    CleverJoker,
    DeviousJoker,
    CraftyJoker,
    HalfJoker,
    CreditCard,
    Banner,
    MysticSummit,
    EightBall,
    Misprint,
    RaisedFist,
    ChaostheClown,
    ScaryFace,
    AbstractJoker,
    DelayedGratification,
    GrosMichel,
    EvenSteven,
    OddTodd,
    Scholar,
    BusinessCard,
    Supernova,
    RideTheBus,
    Egg,
    Runner,
    IceCream,
    Splash,
    BlueJoker,
    FacelessJoker,
    GreenJoker,
    Superposition,
    ToDoList,
    Cavendish,
    RedCard,
    SquareJoker,
    RiffRaff,
    Photograph,
    ReservedParking,
    MailInRebate,
    Hallucination,
    FortuneTeller,
    Juggler,
    Drunkard,
    GoldenJoker,
    Popcorn,
    WalkieTalkie,
    SmileyFace,
    GoldenTicket,
    Swashbuckler,
    HangingChad,
    ShootTheMoon,
    // Uncommon jokers
    JokerStencil,
    FourFingers,
    Mime,
    CeremonialDagger,
    MarbleJoker,
    LoyaltyCard,
    Dusk,
    Fibonacci,
    SteelJoker,
    Hack,
    Pareidolia,
    SpaceJoker,
    Burglar,
    Blackboard,
    SixthSense,
    Constellation,
    Hiker,
    CardSharp,
    Madness,
    Seance,
    Vampire,
    Shortcut,
    Hologram,
    Cloud9,
    Rocket,
    MidasMask,
    Luchador,
    GiftCard,
    TurtleBean,
    Erosion,
    ToTheMoon,
    StoneJoker,
    LuckyCat,
    Bull,
    DietCola,
    TradingCard,
    FlashCard,
    SpareTrousers,
    Ramen,
    Seltzer,
    Castle,
    MrBones,
    Acrobat,
    SockAndBuskin,
    Troubadour,
    Certificate,
    SmearedJoker,
    Throwback,
    RoughGem,
    Bloodstone,
    Arrowhead,
    OnyxAgate,
    GlassJoker,
    Showman,
    FlowerPot,
    MerryAndy,
    OopsAllSixes,
    TheIdol,
    SeeingDouble,
    Matador,
    Satellite,
    Cartomancer,
    Astronomer,
    Bootstraps,
    // Rare jokers
    DNA,
    Vagabond,
    Baron,
    Obelisk,
    BaseballCard,
    AncientJoker,
    Campfire,
    Blueprint,
    WeeJoker,
    HitTheRoad,
    TheDuo,
    TheTrio,
    TheFamily,
    TheOrder,
    TheTribe,
    Stuntman,
    InvisibleJoker,
    Brainstorm,
    DriversLicense,
    BurntJoker,
    // Legendary jokers
    Canio,
    Triboulet,
    Yorick,
    Chicot,
    Perkeo,
}

impl ItemChoice for JokerTypes {
    fn retry(&self) -> bool {
        false
    }

    fn locked(&self) -> bool {
        false
    }
}
