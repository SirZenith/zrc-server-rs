use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use warp::http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use chrono::offset::Utc;
use super::*;

const BEARER: &str = "Bearer ";
const JWT_SECRET: &[u8] = b"secret";
const ENCODING_ALG: jsonwebtoken::Algorithm = jsonwebtoken::Algorithm::HS512;

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: isize, // user id
    exp: usize,
}

pub fn create_jwt(user_id: isize) -> Result<String, ZrcSVError> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::seconds(60))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user_id,
        exp: expiration as usize,
    };
    let header = Header::new(ENCODING_ALG);
    encode(&header, &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|_| ZrcSVError::JWTTokenCreationError)
}

pub fn with_auth(no_auth: bool) -> impl Filter<Extract = (isize,), Error = warp::Rejection> + Clone {
    if no_auth {
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
    match jwt_from_header(&headers) {
        Ok(jwt) => {
            let decoded = decode::<Claims>(
                &jwt,
                &DecodingKey::from_secret(JWT_SECRET),
                &Validation::new(ENCODING_ALG),
            )
            .map_err(|_| warp::reject::custom(ZrcSVError::InvalidAuthToken))?;

            Ok(decoded.claims.sub)
        }
        Err(e) => return Err(warp::reject::custom(e)),
    }
}

fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String, ZrcSVError> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(ZrcSVError::NoAuthHeader),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(ZrcSVError::NoAuthHeader),
    };
    let jwt = match auth_header.strip_prefix(BEARER) {
        Some(jwt) => jwt,
        None => return Err(ZrcSVError::InvalidAuthToken),
    };
    Ok(jwt.to_owned())
}