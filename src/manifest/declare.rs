use camino::Utf8PathBuf;
use smol_str::SmolStr;

pub struct DeclareToml {
    pub tag: Option<SmolStr>,
    pub name: Option<SmolStr>,
    pub path: Option<Utf8PathBuf>,
    pub file_name: Option<SmolStr>,
}


