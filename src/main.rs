mod server;
mod simple_web_server;
use crate::simple_web_server::SimpleWebServer;
use std::thread;
use std::time::Duration;
use crate::server::Settings;

fn main() {
    let settings = Settings {
        port: 8888,
        path: "C:/Users",
        local_network: false,
        spa: false,
        rewrite_to: "",
        index: false,
        directory_listing: true,
        exclude_dot_html: false,
        ipv6: false,
        hidden_dot_files: true,
        cors: false,
        upload: false,
        replace: false,
        delete: false,
        hidden_dot_files_directory_listing: true,
        custom500: "",
        custom404: "",
        custom403: "",
        custom401: "",
        http_auth: false,
        http_auth_username: "く",
        http_auth_password: "password",
        https: true,
        https_cert: r#"-----BEGIN CERTIFICATE-----
            MIIC5jCCAk+gAwIBAgIBATANBgkqhkiG9w0BAQsFADCBmjEzMDEGA1UEAxMqV2Vi
            U2VydmVyRm9yQ2hyb21lMjAyMy0xMC0wOVQwMzo1NDowMS45MjNaMQswCQYDVQQG
            EwJVUzEQMA4GA1UECBMHdGVzdC1zdDEaMBgGA1UEBxMRU2ltcGxlIFdlYiBTZXJ2
            ZXIxGjAYBgNVBAoTEVNpbXBsZSBXZWIgU2VydmVyMQwwCgYDVQQLEwNXU0MwHhcN
            MjMxMDA5MDM1NDAxWhcNMzMxMDA5MDM1NDAxWjCBmjEzMDEGA1UEAxMqV2ViU2Vy
            dmVyRm9yQ2hyb21lMjAyMy0xMC0wOVQwMzo1NDowMS45MjNaMQswCQYDVQQGEwJV
            UzEQMA4GA1UECBMHdGVzdC1zdDEaMBgGA1UEBxMRU2ltcGxlIFdlYiBTZXJ2ZXIx
            GjAYBgNVBAoTEVNpbXBsZSBXZWIgU2VydmVyMQwwCgYDVQQLEwNXU0MwgZ8wDQYJ
            KoZIhvcNAQEBBQADgY0AMIGJAoGBAN4krtbb1Oo2JuFBegljNFFqELsvzG9r1Q46
            pIybMZvHTWCbuo701Sw36+9EJsSoBGK5f6sBvnWs5UfLx2ZFSj07+xTSyzmDyzB/
            ON0zZkDCmMhRyq6bzI4CruO/iivur2HftfcRgzRt4mnlKc19VNJxC+fm2pDypUWG
            M3iiqjLlAgMBAAGjOjA4MAwGA1UdEwQFMAMBAf8wCwYDVR0PBAQDAgL0MBsGA1Ud
            EQQUMBKGEGh0dHA6Ly9sb2NhbGhvc3QwDQYJKoZIhvcNAQELBQADgYEAHf0W58t/
            wehzoHQ2/ytawyD8/wprGQKqYx6ykzbI+5jiEPkFLxIMZOo0F/x7Vx1vrDnyHVx8
            ron2zw0l4cbwA0eZ3TJS4KCHzGqYGDqA06WgUir15pMe914tkK/pTxG4SK7/1pk4
            IA+r0DFQodul1TLEPSYWFPzJo6uT88XCQKQ=
            -----END CERTIFICATE-----"#,
        https_key: r#"-----BEGIN RSA PRIVATE KEY-----
            MIICXQIBAAKBgQDeJK7W29TqNibhQXoJYzRRahC7L8xva9UOOqSMmzGbx01gm7qO
            9NUsN+vvRCbEqARiuX+rAb51rOVHy8dmRUo9O/sU0ss5g8swfzjdM2ZAwpjIUcqu
            m8yOAq7jv4or7q9h37X3EYM0beJp5SnNfVTScQvn5tqQ8qVFhjN4oqoy5QIDAQAB
            AoGAIR0JvPhxproMwKOF1qlljpnyGt0XVdKWpJ4u53to1oZL1/ykFL7qme1Pa6VW
            kWuplQfUFUCCIhYe6r1gdgkadXraHUzJMJNz1pxzZW39W+xe/lOEl/TrNh9ZSVMF
            Njb1psYAl9pGRcfw7EHeP0zhf6aKoJkIbFC3Xl8jNkVCw08CQQDyjWkt2I4tbnnR
            5kItreh07F6Lhk0FHpS1UVZo690phM6mkbr556iqxINTdF9z2QJDyZwDrc3QWunP
            qWojE4QbAkEA6nWaKxP1Hi0/ATsGN2obrlvPe+9F0d8+CTwvMq1r+JaTq0WP+193
            JF3W+5SrVw0Q7HXH8iefDytOuVdM1y6U/wJBAKoBYdpHcgf36hyr5nC79zWUwwPK
            Y0uWTqbz1rv9res+8dUgScyFidv/lwi0hX7eeM7ojZiqhppmToFF/mWNdUcCQF2Q
            CrLQJwwg0DjEfimU/XDqEHWLuZgT92SmEMuvaxvrswgxHVEZ+qiXjhgdbvaxLyS9
            p8nZx9680JCj5vUkEK8CQQCtRZ/dV/UnURJaehgRbjONMGAmBssiRt5Lro/7X+OH
            TybM+Qvhza1sMaINRex2nTdgscW5IMFQuTzvHWRaWF4A
            -----END RSA PRIVATE KEY-----"#,
    };
    let mut server = SimpleWebServer::new(settings);
    println!("Server started: {}", server.start());
    //let mut i = 0;
    loop {
    //    i += 1;
    //    if i > 20 {
    //        server.terminate();
    //    }
        thread::sleep(Duration::from_millis(100));
    }
}

