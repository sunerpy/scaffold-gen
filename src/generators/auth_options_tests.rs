use super::*;

#[test]
fn parse_round_trip_for_both_modes() {
    for mode in [AuthMode::None, AuthMode::Jwt, AuthMode::AzureAd] {
        let parsed = AuthMode::parse_from_str(mode.as_str());
        assert_eq!(parsed, Some(mode));
    }
}

#[test]
fn as_str_values() {
    assert_eq!(AuthMode::None.as_str(), "none");
    assert_eq!(AuthMode::Jwt.as_str(), "jwt");
    assert_eq!(AuthMode::AzureAd.as_str(), "azure-ad");
}

#[test]
fn is_enabled_flag() {
    assert!(AuthMode::Jwt.is_enabled());
    assert!(AuthMode::AzureAd.is_enabled());
    assert!(!AuthMode::None.is_enabled());
}

#[test]
fn is_azure_ad_flag() {
    assert!(AuthMode::AzureAd.is_azure_ad());
    assert!(!AuthMode::Jwt.is_azure_ad());
    assert!(!AuthMode::None.is_azure_ad());
}

#[test]
fn parse_azure_ad_aliases() {
    assert_eq!(
        AuthMode::parse_from_str("azure-ad"),
        Some(AuthMode::AzureAd)
    );
    assert_eq!(AuthMode::parse_from_str("azuread"), Some(AuthMode::AzureAd));
    assert_eq!(
        AuthMode::parse_from_str("azure_ad"),
        Some(AuthMode::AzureAd)
    );
    assert_eq!(
        AuthMode::parse_from_str("AZURE-AD"),
        Some(AuthMode::AzureAd)
    );
    assert_eq!(AuthMode::parse_from_str("AzureAd"), Some(AuthMode::AzureAd));
}

#[test]
fn default_is_none() {
    assert_eq!(AuthMode::default(), AuthMode::None);
}

#[test]
fn parse_is_case_insensitive() {
    assert_eq!(AuthMode::parse_from_str("JWT"), Some(AuthMode::Jwt));
    assert_eq!(AuthMode::parse_from_str("None"), Some(AuthMode::None));
}

#[test]
fn parse_bogus_is_none() {
    assert!(AuthMode::parse_from_str("bogus").is_none());
}
