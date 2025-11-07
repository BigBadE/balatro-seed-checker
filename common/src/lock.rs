use alloc::string::String;
use alloc::vec::Vec;

pub struct Lock {
    locked: Vec<String>,
    unlocked: Vec<String>,
}

impl Default for Lock { fn default() -> Self { Self { locked: Vec::new(), unlocked: Vec::new() } } }
impl Lock {
    pub fn new() -> Self { Self::default() }

    fn push_unique(vec: &mut Vec<String>, name: &str) {
        if !vec.iter().any(|s| s.as_str() == name) { vec.push(String::from(name)); }
    }

    pub fn lock<S: AsRef<str>>(&mut self, name: S) { Self::push_unique(&mut self.locked, name.as_ref()); }
    pub fn lock_many<'a, I: IntoIterator<Item=&'a str>>(&mut self, names: I) { for n in names { self.lock(n); } }
    pub fn unlock<S: AsRef<str>>(&mut self, name: S) { Self::push_unique(&mut self.unlocked, name.as_ref()); }
    pub fn unlock_many<'a, I: IntoIterator<Item=&'a str>>(&mut self, names: I) { for n in names { self.unlock(n); } }

    pub fn is_locked<S: AsRef<str>>(&self, name: S) -> bool {
        let n = name.as_ref();
        if self.unlocked.iter().any(|s| s.as_str() == n) { return false; }
        self.locked.iter().any(|s| s.as_str() == n)
    }

    pub fn init_locks(&mut self, ante: i32, fresh_profile: bool, fresh_run: bool) {
        if ante < 2 {
            self.lock_many([
                "The Mouth","The Fish","The Wall","The House","The Mark","The Wheel","The Arm","The Water","The Needle","The Flint",
            ].iter().copied());
            self.lock_many([
                "Standard Tag","Meteor Tag","Buffoon Tag","Handy Tag","Garbage Tag","Ethereal Tag","Top-up Tag","Orbital Tag",
            ].iter().copied());
        }
        if ante < 3 { self.lock_many(["The Tooth","The Eye"].iter().copied()); }
        if ante < 4 { self.lock("The Plant"); }
        if ante < 5 { self.lock("The Serpent"); }
        if ante < 6 { self.lock("The Ox"); }

        if fresh_profile {
            self.lock_many([
                "Negative Tag","Foil Tag","Holographic Tag","Polychrome Tag","Rare Tag",
                "Golden Ticket","Mr. Bones","Acrobat","Sock and Buskin","Swashbuckler","Troubadour",
                "Certificate","Smeared Joker","Throwback","Hanging Chad","Rough Gem","Bloodstone","Arrowhead","Onyx Agate","Glass Joker",
                "Showman","Flower Pot","Blueprint","Wee Joker","Merry Andy","Oops! All 6s","The Idol",
                "Seeing Double","Matador","Hit the Road","The Duo","The Trio","The Family","The Order","The Tribe",
                "Stuntman","Invisible Joker","Brainstorm","Satellite","Shoot the Moon","Driver's License","Cartomancer","Astronomer","Burnt Joker","Bootstraps",
                "Overstock Plus","Liquidation","Glow Up","Reroll Glut","Omen Globe","Observatory","Nacho Tong","Recyclomancy","Tarot Tycoon","Planet Tycoon","Money Tree","Antimatter","Illusion","Petroglyph","Retcon","Palette",
            ].iter().copied());
        }

        if fresh_run {
            self.lock_many([
                "Planet X","Ceres","Eris",
                "Stone Joker","Steel Joker","Glass Joker","Golden Ticket","Lucky Cat",
                "Cavendish","Overstock Plus","Liquidation","Glow Up","Reroll Glut","Omen Globe","Observatory","Nacho Tong","Recyclomancy","Tarot Tycoon","Planet Tycoon","Money Tree","Antimatter","Illusion","Petroglyph","Retcon","Palette",
            ].iter().copied());
        }
    }

    pub fn init_unlocks(&mut self, ante: i32, fresh_profile: bool) {
        if ante == 2 {
            self.unlock_many([
                "The Mouth","The Fish","The Wall","The House","The Mark","The Wheel","The Arm","The Water","The Needle","The Flint",
            ].iter().copied());
            self.unlock_many([
                "Standard Tag","Meteor Tag","Buffoon Tag","Handy Tag","Garbage Tag","Ethereal Tag","Top-up Tag","Orbital Tag",
            ].iter().copied());
            if !fresh_profile { self.unlock("Negative Tag"); }
        }
        if ante == 3 { self.unlock_many(["The Tooth","The Eye"].iter().copied()); }
        if ante == 4 { self.unlock("The Plant"); }
        if ante == 5 { self.unlock("The Serpent"); }
        if ante == 6 { self.unlock("The Ox"); }
    }

    pub fn handle_selected_unlocks<'a, I: IntoIterator<Item=&'a str>>(&mut self, selected: I) { for n in selected { self.unlock(n); } }
    pub fn lock_level_two_vouchers(&mut self) {
        // Matches Blueprint Lock.firstLock
        self.lock_many([
            "Overstock Plus", "Liquidation", "Glow Up", "Reroll Glut", "Omen Globe",
            "Observatory", "Nacho Tong", "Recyclomancy", "Tarot Tycoon", "Planet Tycoon",
            "Money Tree", "Antimatter", "Illusion", "Petroglyph", "Retcon", "Palette",
        ].iter().copied());
    }
}
