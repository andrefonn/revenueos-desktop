use url::Url;

pub const APP_ORIGIN: &str = "https://www.revenueos.app.br";
pub const UPDATE_ORIGIN: &str = "https://github.com/andrefonn/revenueos-desktop/releases/download/";

pub fn is_app(url: &Url) -> bool {
    is_safe_external(url)
        && matches!(
            url.host_str(),
            Some("www.revenueos.app.br" | "revenueos.app.br")
        )
}
pub fn is_meta(url: &Url) -> bool {
    is_safe_external(url)
        && matches!(
            url.host_str(),
            Some("www.facebook.com" | "web.facebook.com")
        )
}
pub fn is_local(url: &Url) -> bool {
    without_credentials(url)
        && url.port().is_none()
        && matches!(
            (url.scheme(), url.host_str()),
            ("tauri", Some("localhost")) | ("http" | "https", Some("tauri.localhost"))
        )
}
pub fn is_safe_external(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str().is_some()
        && url.port_or_known_default() == Some(443)
        && without_credentials(url)
}
pub fn is_update(url: &Url) -> bool {
    if !is_safe_external(url)
        || url.host_str() != Some("github.com")
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return false;
    }

    let Some(asset) = url
        .path()
        .strip_prefix("/andrefonn/revenueos-desktop/releases/download/")
    else {
        return false;
    };
    let Some((tag, filename)) = asset.split_once('/') else {
        return false;
    };
    if filename != "RevenueOS-Windows-x64-Setup.exe" {
        return false;
    }
    let Some(version) = tag.strip_prefix('v') else {
        return false;
    };
    let parts: Vec<_> = version.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (part.len() == 1 || !part.starts_with('0'))
        })
}

fn without_credentials(url: &Url) -> bool {
    url.username().is_empty() && url.password().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn u(value: &str) -> Url {
        Url::parse(value).unwrap()
    }

    #[test]
    fn app_requires_exact_https_origin_without_userinfo() {
        assert!(is_app(&u("https://revenueos.app.br/dashboard")));
        assert!(is_app(&u("https://www.revenueos.app.br/dashboard")));
        assert!(is_app(&u(
            "https://www.revenueos.app.br:443/inbox?tab=all#latest"
        )));
        for input in [
            "http://revenueos.app.br",
            "https://revenueos.app.br.evil.test",
            "https://revenueos.app.br:444",
            "https://evil@revenueos.app.br",
            "https://revenueos.app.br@evil.test",
            "https://www.revenueos.app.br.evil.test",
            "https://app.revenueos.app.br",
            "https://revenueos.app.br.",
            "https://:password@www.revenueos.app.br",
        ] {
            assert!(!is_app(&u(input)), "{input}");
        }
    }
    #[test]
    fn meta_allowlist_is_exact() {
        assert!(is_meta(&u("https://www.facebook.com/v24.0/dialog/oauth")));
        assert!(is_meta(&u("https://web.facebook.com/dialog/oauth")));
        assert!(is_meta(&u(
            "https://www.facebook.com:443/dialog/oauth?client_id=123"
        )));
        for input in [
            "https://facebook.com.evil.test",
            "https://www.facebook.com:444",
            "http://www.facebook.com/dialog/oauth",
            "https://facebook.com/dialog/oauth",
            "https://m.facebook.com/dialog/oauth",
            "https://evil@web.facebook.com/dialog/oauth",
            "https://:password@www.facebook.com/dialog/oauth",
        ] {
            assert!(!is_meta(&u(input)), "{input}");
        }
    }
    #[test]
    fn local_assets_do_not_include_arbitrary_localhost_ports() {
        assert!(is_local(&u("tauri://localhost/index.html")));
        assert!(is_local(&u("http://tauri.localhost/index.html")));
        assert!(is_local(&u("https://tauri.localhost/index.html")));
        for input in [
            "http://localhost:3000",
            "file:///C:/Windows/system.ini",
            "tauri://localhost:3000/index.html",
            "tauri://tauri.localhost/index.html",
            "https://tauri.localhost:444/index.html",
            "http://tauri.localhost:8080/index.html",
            "https://tauri.localhost.evil.test/index.html",
            "tauri://localhost.evil.test/index.html",
            "tauri://user@localhost/index.html",
            "http://user:password@tauri.localhost/index.html",
            "ftp://tauri.localhost/index.html",
        ] {
            assert!(!is_local(&u(input)), "{input}");
        }
    }
    #[test]
    fn external_links_never_execute_protocols_or_credentials() {
        assert!(is_safe_external(&u("https://docs.asaas.com/")));
        assert!(is_safe_external(&u(
            "https://example.com:443/help?q=example#topic"
        )));
        for input in [
            "file:///C:/x.exe",
            "javascript:alert(1)",
            "ms-msdt:/id",
            "http://example.com",
            "https://user:pass@example.com",
            "https://:pass@example.com",
            "https://example.com:8443",
        ] {
            assert!(!is_safe_external(&u(input)), "{input}");
        }
    }
    #[test]
    fn updater_only_accepts_versioned_assets_from_our_release_repo() {
        assert!(is_update(&u("https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe")));
        assert!(is_update(&u("https://github.com:443/andrefonn/revenueos-desktop/releases/download/v12.34.56/RevenueOS-Windows-x64-Setup.exe")));
        for input in [
            "https://github.com/attacker/revenueos-desktop/releases/download/v0.1.1/a.exe",
            "http://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/a.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/../a.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/latest/download/a.exe",
            "https://github.com.evil.test/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe",
            "https://github.com:444/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe",
            "https://user@github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe",
            "https://:password@github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe?download=1",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe#asset",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe?",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe#",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe/extra",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/RevenueOS-Windows-x64-Setup.exe/",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/0.1.1/RevenueOS-Windows-x64-Setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1/RevenueOS-Windows-x64-Setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1.2/RevenueOS-Windows-x64-Setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0..1/RevenueOS-Windows-x64-Setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v01.1.1/RevenueOS-Windows-x64-Setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1-beta/RevenueOS-Windows-x64-Setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1%2Fextra/RevenueOS-Windows-x64-Setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download/v0.1.1/%52evenueOS-Windows-x64-Setup.exe",
            "https://github.com/andrefonn/revenueos-desktop/releases/download//v0.1.1/RevenueOS-Windows-x64-Setup.exe",
        ] {
            assert!(!is_update(&u(input)), "{input}");
        }
    }
}
