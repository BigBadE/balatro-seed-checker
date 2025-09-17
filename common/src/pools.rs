#[cfg(feature = "std")]
use strum::IntoEnumIterator;
use crate::items::{JokerTypes, RandomSource, Spectral};
#[cfg(feature = "std")]
use crate::items::{Editions, Joker, JokerRarity, Planets, Tarots};

use crate::items::JokerTypes::*;
use crate::random::Random;

#[cfg(feature = "std")]
pub fn next_joker(random: &mut Random, source: RandomSource, ante: i32, has_stickers: bool) -> Joker {
    // Get rarity
    let rarity = match source {
        RandomSource::Soul => JokerRarity::Legendary,
        RandomSource::Wraith => JokerRarity::Rare,
        RandomSource::RareTag => JokerRarity::Rare,
        RandomSource::UncommonTag => JokerRarity::Uncommon,
        _ => {
            match random.random(&format!("rarity{ante}{source}")) {
                x if x > 0.95 => JokerRarity::Rare,
                x if x > 0.7 => JokerRarity::Uncommon,
                _ => JokerRarity::Common,
            }
        }
    };

    // Get edition
    let edition_rate = 1.0f64;
    // TODO glow up/hone vouchers
    let edition = match random.random(&format!("edi{source}{ante}")) {
        x if x > 0.997 => Editions::Negative,
        x if x > 1.0 - 0.006 * edition_rate => Editions::Polychrome,
        x if x > 1.0 - 0.02 * edition_rate => Editions::Holographic,
        x if x > 1.0 - 0.04 * edition_rate => Editions::Foil,
        _ => Editions::None,
    };

    // Get next joker
    let joker = *match rarity {
        JokerRarity::Legendary => random.rand_choice("Joker4", &LEGENDARY_JOKERS),
        JokerRarity::Rare => random.rand_choice(&format!("Joker3{source}{ante}"), &RARE_JOKERS),
        JokerRarity::Uncommon => random.rand_choice(&format!("Joker2{source}{ante}"), &UNCOMMON_JOKERS),
        JokerRarity::Common => random.rand_choice(&format!("Joker1{source}{ante}"), &COMMON_JOKERS),
    };

    // Get next joker stickers
    let stickers = vec![false; 3];
    if has_stickers {
        /* TODO stickers
        let _stickers = random.random(&format!("{ante}{}", match source {
            RandomSource::BuffonPack => "packetper",
            _ => "etperpoll"
        }));
        if (stickerPoll > 0.7 && (params.stake == "Black Stake" || params.stake == "Blue Stake" || params.stake == "Purple Stake" || params.stake == "Orange Stake" || params.stake == "Gold Stake")) {
            if (joker != "Gros Michel" && joker != "Ice Cream" && joker != "Cavendish" && joker != "Luchador"
                && joker != "Turtle Bean" && joker != "Diet Cola" && joker != "Popcorn" && joker != "Ramen"
                && joker != "Seltzer" && joker != "Mr. Bones" && joker != "Invisible Joker") {
                stickers.eternal = true;
            }
        }
        if ((stickerPoll > 0.4 && stickerPoll <= 0.7) && (params.stake == "Orange Stake" || params.stake == "Gold Stake")) {
            if (joker != "Ceremonial Dagger" && joker != "Ride the Bus" && joker != "Runner" && joker != "Constellation"
                && joker != "Green Joker" && joker != "Red Card" && joker != "Madness" && joker != "Square Joker"
                && joker != "Vampire" && joker != "Rocket" && joker != "Obelisk" && joker != "Lucky Cat"
                && joker != "Flash Card" && joker != "Spare Trousers" && joker != "Castle" && joker != "Wee Joker") {
                stickers.perishable = true;
            }
        }

        if (params.stake == "Gold Stake") {
            stickers.rental = random(((source == "buf")? "packssjr": "ssjr") + anteStr) > 0.7;
        }*/
    }

    Joker {
        joker,
        rarity,
        edition,
        stickers,
    }
}

#[cfg(feature = "std")]
pub fn next_tarot(random: &mut Random, source: RandomSource, ante: i32, soulable: bool) -> Tarots {
    if soulable && random.random(&format!("soul_Tarot{ante}")) > 0.997 {
        return Tarots::TheSoul;
    }
    *random.rand_choice(&format!("Tarot{source}{ante}"), Tarots::iter().collect::<Vec<_>>().as_slice())
}

#[cfg(feature = "std")]
pub fn next_planet(random: &mut Random, source: RandomSource, ante: i32, soulable: bool) -> Planets {
    if soulable && random.random(&format!("soul_Planet{ante}")) > 0.997 {
        return Planets::BlackHole;
    }
    *random.rand_choice(&format!("Planet{source}{ante}"), Planets::iter().collect::<Vec<_>>().as_slice())
}

pub fn next_spectral(_random: &mut Random, _source: RandomSource, _ante: i32, _soulable: bool) -> Spectral {
    todo!()
}

pub const COMMON_JOKERS: [JokerTypes; 61] = [
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
];

pub const UNCOMMON_JOKERS: [JokerTypes; 64] = [
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
];

pub const RARE_JOKERS: [JokerTypes; 20] = [
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
];

pub const LEGENDARY_JOKERS: [JokerTypes; 5] = [
    Canio,
    Triboulet,
    Yorick,
    Chicot,
    Perkeo,
];