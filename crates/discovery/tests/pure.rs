use std::collections::BTreeSet;

use discovery::testing::{dlcs_from_appids, edition_from_appids, parse_save_filename};
use discovery::{Dlc, Edition, SavePrefix};

fn set(ids: &[u32]) -> BTreeSet<u32> {
    ids.iter().copied().collect()
}

#[test]
fn edition_is_highest_owned_dlc() {
    assert_eq!(edition_from_appids(&set(&[])), Edition::Rebirth);
    assert_eq!(edition_from_appids(&set(&[401920])), Edition::Afterbirth);
    assert_eq!(
        edition_from_appids(&set(&[401920, 570660])),
        Edition::AfterbirthPlus
    );
    assert_eq!(
        edition_from_appids(&set(&[401920, 570660, 1426300])),
        Edition::Repentance
    );
    assert_eq!(
        edition_from_appids(&set(&[401920, 570660, 1426300, 3353470])),
        Edition::RepentancePlus
    );
    // Gaps in the sequence: it still counts the highest one present.
    assert_eq!(
        edition_from_appids(&set(&[3353470])),
        Edition::RepentancePlus
    );
}

#[test]
fn dlcs_lists_all_owned_in_order() {
    assert_eq!(dlcs_from_appids(&set(&[])), Vec::<Dlc>::new());
    assert_eq!(
        dlcs_from_appids(&set(&[401920, 570660, 1426300, 3353470])),
        vec![
            Dlc::Afterbirth,
            Dlc::AfterbirthPlus,
            Dlc::Repentance,
            Dlc::RepentancePlus
        ]
    );
}

#[test]
fn parses_valid_save_filenames() {
    assert_eq!(
        parse_save_filename("rep+persistentgamedata1.dat"),
        Some((SavePrefix::RepPlus, 1))
    );
    assert_eq!(
        parse_save_filename("rep_persistentgamedata2.dat"),
        Some((SavePrefix::Rep, 2))
    );
    assert_eq!(
        parse_save_filename("rep+persistentgamedata3.dat"),
        Some((SavePrefix::RepPlus, 3))
    );
}

#[test]
fn rejects_non_save_filenames() {
    assert_eq!(
        parse_save_filename("20250626.rep+persistentgamedata1.dat"),
        None
    );
    assert_eq!(parse_save_filename("options.ini"), None);
    assert_eq!(parse_save_filename("rep+persistentgamedata.dat"), None);
    assert_eq!(parse_save_filename("rep+persistentgamedata12.dat"), None);
    assert_eq!(parse_save_filename("rep+persistentgamedata0.dat"), None);
    assert_eq!(parse_save_filename("persistentgamedata1.dat"), None);
}

const REAL_ACF: &str = r#"
"AppState"
{
	"appid"		"250900"
	"name"		"The Binding of Isaac: Rebirth"
	"installdir"		"The Binding of Isaac Rebirth"
	"InstalledDepots"
	{
		"250902"
		{
			"manifest"		"4994611894646808503"
			"size"		"326498528"
		}
		"250905"
		{
			"manifest"		"1709017229885880564"
			"size"		"165283102"
			"dlcappid"		"401920"
		}
		"250908"
		{
			"manifest"		"7333987924869605149"
			"size"		"147989670"
			"dlcappid"		"570660"
		}
		"250911"
		{
			"manifest"		"7652847940910762229"
			"size"		"655313480"
			"dlcappid"		"1426300"
		}
		"3353471"
		{
			"manifest"		"229910742625134068"
			"size"		"763003388"
			"dlcappid"		"3353470"
		}
	}
}
"#;

#[test]
fn parses_installdir_and_dlc_appids_from_real_acf() {
    let (installdir, dlcs) =
        discovery::testing::parse_manifest_fields(REAL_ACF).expect("valid manifest");
    assert_eq!(installdir, "The Binding of Isaac Rebirth");
    assert_eq!(
        dlcs,
        [401920u32, 570660, 1426300, 3353470].into_iter().collect()
    );
}

#[test]
fn malformed_acf_returns_none() {
    assert_eq!(
        discovery::testing::parse_manifest_fields("questo non è vdf {{{"),
        None
    );
}
