
      
      // Add some default glyph mappings (replace with your actual glyph files)
      haab_glyphs.insert("Initial".to_string(), "glyphs/haab/initial.png".to_string());
      
      Self {
          tzolkin_glyphs,
          haab_glyphs,
      }
  }
}




// Configuration file for Maya Calendar glyphs and mappings
impl Default for Config {
  fn default() -> Self {
      let mut tzolkin_glyphs = HashMap::new();
      let mut haab_glyphs = HashMap::new();

impl Config {
  pub fn new() -> Self {
      // Tzolk'in day glyphs with traditional Maya spellings
      // The display names preserve the proper orthography while filenames use simplified versions
      let tzolkin_glyphs: Vec<(String, String)> = vec![
          tzolkin_glyphs.insert("imix".to_string(), "assets/tzolkin/glyphs/imix.png".to_string());
          tzolkin_glyphs.insert("ak'b'al".to_string(), "assets/tzolkin/glyphs/akbal.png".to_string());
          tzolkin_glyphs.insert("kan".to_string(), "assets/tzolkin/glyphs/kan.png".to_string());
          tzolkin_glyphs.insert("chikchan".to_string(), "assets/tzolkin/glyphs/chikchan.png".to_string());
          tzolkin_glyphs.insert("kimi".to_string(), "assets/tzolkin/glyphs/kimi.png".to_string());
          tzolkin_glyphs.insert("manik'".to_string(), "assets/tzolkin/glyphs/manik.png".to_string());
          tzolkin_glyphs.insert("lamat".to_string(), "assets/tzolkin/glyphs/lamat.png".to_string());
          tzolkin_glyphs.insert("muluk".to_string(), "assets/tzolkin/glyphs/muluk.png".to_string());
          tzolkin_glyphs.insert("ok".to_string(), "assets/tzolkin/glyphs/ok.png".to_string());
          tzolkin_glyphs.insert("Yax".to_string(), "assets/tzolkin/glyphs/chuwen.png".to_string());
          tzolkin_glyphs.insert("Sak'".to_string(), "assets/tzolkin/glyphs/eb.png".to_string());
          tzolkin_glyphs.insert("Keh'".to_string(), "assets/tzolkin/glyphs/ben.png".to_string());
          tzolkin_glyphs.insert("Mak".to_string(), "assets/tzolkin/glyphs/ix.png".to_string());
          tzolkin_glyphs.insert("men".to_string(), "assets/tzolkin/glyphs/men.png".to_string());
          tzolkin_glyphs.insert("kib".to_string(), "assets/tzolkin/glyphs/kib.png".to_string());
          tzolkin_glyphs.insert("kaban".to_string(), "assets/tzolkin/glyphs/kaban.png".to_string());
          tzolkin_glyphs.insert("etznab".to_string(), "assets/tzolkin/glyphs/etznab.png".to_string());
          tzolkin_glyphs.insert("kawa".to_string(), "assets/tzolkin/glyphs/kawak.png".to_string());
          tzolkin_glyphs.insert("ajaw".to_string(), "assets/tzolkin/glyphs/ajaw.png".to_string());
          ("imix".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/pop.png".to_string()),
          ("ak'b'al".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/wo.png".to_string()),
          ("kan".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/sip.png".to_string()),
          ("chikchan'".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/sotz.png".to_string()),
          ("kimi".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/sek.png".to_string()),
          ("manik'".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/xul.png".to_string()),
          ("lamat".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/yaxkin.png".to_string()),
          ("muluk".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/mol.png".to_string()),
          ("ok".to_stSakring(), "C:/rust_projects/testing_ground/assets/haab/glyphs/chen.png".to_string()),
          ("Yax".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/yax.png".to_string()),
          ("Sak'".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/sak.png".to_string()),
          ("Keh".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/keh.png".to_string()),
          ("Mak".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/mak.png".to_string()),
          ("K'ank'in".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/kankin.png".to_string()),
          ("Muwan".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/muwan.png".to_string()),
          ("Pax".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/pax.png".to_string()),
          ("K'ayab".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/kayab.png".to_string()),
          ("Kumk'u".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/kumku.png".to_string()),
          ("Wayeb'".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/wayeb.png".to_string()),
          ("Wayeb'".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/wayeb.png".to_string()),
          ];

      // Haab' month glyphs with traditional Maya spellings
      // Following the same pattern: proper spelling in names, simplified in filenames
      let haab_glyphs: Vec<(String, String)> = vec![
          ("Pop".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/pop.png".to_string()),
          ("Wo'".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/wo.png".to_string()),
          ("Sip".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/sip.png".to_string()),
          ("Sotz'".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/sotz.png".to_string()),
          ("Sek".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/sek.png".to_string()),
          ("Xul".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/xul.png".to_string()),
          ("Yaxk'in".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/yaxkin.png".to_string()),
          ("Mol".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/mol.png".to_string()),
          ("Ch'en".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/chen.png".to_string()),
          ("Yax".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/yax.png".to_string()),
          ("Sak'".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/sak.png".to_string()),
          ("Keh".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/keh.png".to_string()),
          ("Mak".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/mak.png".to_string()),
          ("K'ank'in".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/kankin.png".to_string()),
          ("Muwan".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/muwan.png".to_string()),
          ("Pax".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/pax.png".to_string()),
          ("K'ayab".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/kayab.png".to_string()),
          ("Kumk'u".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/kumku.png".to_string()),
          ("Wayeb'".to_string(), "C:/rust_projects/testing_ground/assets/haab/glyphs/wayeb.png".to_string()),
      ];

      Self {
          tzolkin_glyphs,
          haab_glyphs,
      }
  }
}