use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use chrono::offset::Utc;
use warp::http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use super::*;

pub const BASIC: &str = "Basic ";
pub const BEARER: &str = "Bearer ";
const JWT_SECRET: &[u8] = b"secret";
const ENCODING_ALG: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS512;

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: isize, // user id
    exp: usize,
}

pub fn hash_pwd(pwd: &str) -> String {
    let digest = md5::compute(pwd.as_bytes());
    format!("{:x}", digest)
}

fn token_from_header(headers: &HeaderMap<HeaderValue>, token_prefix: &str) -> Result<String, ZrcSVError> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(ZrcSVError::NoAuthHeader),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(ZrcSVError::NoAuthHeader),
    };
    let token = match auth_header.strip_prefix(token_prefix) {
        Some(t) => t,
        None => return Err(ZrcSVError::InvalidToken(format!("can't find token prefix {}", token_prefix))),
    };
    Ok(token.to_owned())
}

pub fn with_basic_auth(is_auth_off: bool, pool: SqlitePool) -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    if is_auth_off {
        warp::any().and_then(blank_basic_auth).boxed()
    } else {
        warp::header::headers_cloned()
            .map(move |headers: HeaderMap<HeaderValue>| headers)
            .and(with_db_access_manager(pool))
            .and_then(basic_authorize)
            .boxed()
    }
}

async fn blank_basic_auth() -> ZrcSVResult<String> {
    Ok("nothing".to_string())
}

async fn basic_authorize(headers: HeaderMap<HeaderValue>, conn: DBAccessManager) -> ZrcSVResult<String> {
    let user_id = match check_basic_token(headers, conn) {
        Ok(id) => id,
        Err(e) => return Err(warp::reject::custom(e)),
    };
    let jwt = match create_jwt(user_id) {
        Ok(t) => t,
        Err(e) => return Err(warp::reject::custom(e)),
    };
    Ok(jwt)
}

fn check_basic_token(headers: HeaderMap<HeaderValue>, conn: DBAccessManager) -> Result<isize, ZrcSVError> {
    let token = token_from_header(&headers, BASIC)?;
    let token_bytes = base64::decode(token.as_bytes())
        .map_err(|e| ZrcSVError::InvalidToken(format!("decoding error, {}", e)))?;
    let token = String::from_utf8_lossy(&token_bytes).to_string();
    log::debug!("decoded basic token: {}", token);
    
    let parts: Vec<&str> = token.split(":").collect();
    if parts.len() != 2 {
        return Err(ZrcSVError::InvalidToken("invalid token format".to_string()))
    }
    let (name, pwd) = (parts[0], parts[1]);
    log::debug!("email/user name: {}, password: {}", name, pwd);
    let pwd_hash = hash_pwd(pwd);
    log::debug!("password hash: {}", pwd_hash);

    
    let user_id = conn.login(name, &pwd_hash).map_err(|e| match e {
        ZrcDBError::DataNotFound(_) => ZrcSVError::UserNotFound,
        _ => ZrcSVError::DBError(e)
    })?;
    Ok(user_id)
}

pub fn create_jwt(user_id: isize) -> Result<String, ZrcSVError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(10))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        exp: expiration as usize,
    };
    let header = Header::new(ENCODING_ALG);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET)).map_err(|_| ZrcSVError::JWTTokenCreationError)
}

pub fn with_auth(is_auth_off: bool) -> impl Filter<Extract = (isize,), Error = warp::Rejection> + Clone {
    if is_auth_off {
        warp::any().and_then(blank_auth).boxed()
    } else {
        warp::header::headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| headers)
        .and_then(authorize)
        .boxed()
    }
    
}

async fn blank_auth() -> ZrcSVResult<isize> {
    Ok(STATIC_USER_ID)
}

async fn authorize(headers: HeaderMap<HeaderValue>) -> ZrcSVResult<isize> {
    match token_from_header(&headers, BEARER) {
        Ok(jwt) => {
            let decoded = decode::<Claims>(
                &jwt,
                &DecodingKey::from_secret(JWT_SECRET),
                &Validation::new(ENCODING_ALG),
            )
            .map_err(|e| warp::reject::custom(ZrcSVError::InvalidToken(format!("{}", e))))?;

            Ok(decoded.claims.sub)
        }
        Err(e) => return Err(warp::reject::custom(e)),
    }
}

