use crate::app;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use scrypt::password_hash::rand_core::{OsRng, RngCore};
use sha2::Sha256;
use tracing::log::{debug, error};

/* THE IDEA AND COMPROMISES
 * The approach to implement API token for this application is very basic, and
 * must take into consideration automation that does not have ability to sign
 * the message.
 * One example of usage is iPhone based shortcuts, where we can only parse and
 * construct JSON payload. The shortcut feature does not allow to sign, generate
 * any more sophisticated tokens.
 *
 * The token takes following assumptions:
 *  1. User token is stored in plaintext in user config.
 *  2. Each application uses its own secret key.
 *  3. User token gets signed by app secret and displayed ONCE in user account
 *     settings page.
 *  4. User adds token in header of the request, this allows to verify the source.
 *
 * The token gets invalidated when:
 *  1. User regenerate token.
 *  2. App secret changes.
 */

type HmacSha256 = Hmac<Sha256>;

fn get_secret_mac() -> HmacSha256 {
    HmacSha256::new_from_slice(app::get_secret_key().as_bytes())
        .expect("HMAC can take key of any size")
}

fn base64_decode(payload: &str) -> Vec<u8> {
    match URL_SAFE_NO_PAD.decode(payload) {
        Ok(t) => t,
        Err(err) => {
            error!("Unable to decode base64");
            debug!("{}", err.to_string());
            panic!("Unable to decode base64");
        }
    }
}

fn base64_encode(payload: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(payload)
}

pub fn generate(nbytes: usize) -> String {
    let mut bytes = vec![0u8; nbytes];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(&bytes)
}

pub fn sign(user_token: &str) -> String {
    let user_token = base64_decode(user_token);

    let mut sig = get_secret_mac();
    sig.update(user_token.as_slice());

    base64_encode(&sig.finalize().into_bytes())
}

pub fn verify(payload_token: &str, user_token: &str) -> bool {
    let Ok(payload_bytes) = URL_SAFE_NO_PAD.decode(payload_token) else {
        return false;
    };

    let mut mac = get_secret_mac();
    let Ok(user_bytes) = URL_SAFE_NO_PAD.decode(user_token) else {
        return false;
    };

    mac.update(&user_bytes);
    mac.verify_slice(&payload_bytes).is_ok()
}
