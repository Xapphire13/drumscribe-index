pub fn correct_artist(artist: &str) -> &str {
    match artist {
        "Blink-182" => "blink-182",
        "Born Of Osiris" => "Born of Osiris",
        "Bowling For Soup" => "Bowling for Soup",
        "Bullet For My Valentine" => "Bullet for My Valentine",
        "Fountains Of Wayne" => "Fountains of Wayne",
        "Jackson 5" => "The Jackson 5",
        "Puddle Of Mudd" => "Puddle of Mudd",
        "Rage Against The Machine" => "Rage Against the Machine",
        "System Of A Down" => "System of a Down",
        "The Presidents Of The United States Of America" => {
            "The Presidents of the United States of America"
        }
        _ => artist,
    }
}
