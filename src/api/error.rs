use super::*;
use std::convert::Infallible;
use warp::http::StatusCode;

use crate::data_access::ZrcDBError;

#[allow(dead_code)]
const TRANSICATION_ERROR: i32 = -7; // 处理交易时发生了错误
                                      // const GET_ITEM_FAILED: i32 = -6; // 此物品目前无法获取
#[allow(dead_code)]
const DOWNLOAD_ALREADY_FINISHED: i32 = -5; // 所有的曲目都已经下载完毕
#[allow(dead_code)]
const ACCOUNT_LOGIN_ELSEWHERE: i32 = -4; // 您的账号已在别处登录
                                           // const CONNECT_FAILED: i32 = -3; // 无法连接至服务器
#[allow(dead_code)]
const UNKNOWN_ERROR: i32 = 1; // 发生未知错误
#[allow(dead_code)]
const SERVER_MAINTAINING: i32 = 2; // Arcaea服务器正在维护
#[allow(dead_code)]
const VERSION_TOO_OLD: i32 = 5; // 请更新Arcaea到最新版本

// Authentication -------------------------------------------------------------
#[allow(dead_code)]
const BLOCKED_IP: i32 = 100; // 无法在此ip地址下登录游戏
#[allow(dead_code)]
const USERNAME_DUPLICATED: i32 = 101; // 用户名占用
#[allow(dead_code)]
const EMAIL_DUPLICATED: i32 = 102; // 电子邮箱已注册
#[allow(dead_code)]
const DEVICE_ID_DUPLICATED: i32 = 103; // 已有一个账号由此设备创建
#[allow(dead_code)]
const PWD_DOES_NOT_MATCH: i32 = 104; // 用户名密码错误
#[allow(dead_code)]
const LOGIN_ON_TOO_MUCH_DEVICES: i32 = 105; // 24 小时内登入两台设备
                                              // const ACCOUNT_FROZEN: i32 = 106; // 账户冻结

// World Mode -----------------------------------------------------------------
#[allow(dead_code)]
const LACKING_STAMINA: i32 = 107; // 你没有足够的体力
#[allow(dead_code)]
const EVENT_ENDED: i32 = 113; // 活动已结束
#[allow(dead_code)]
const EVENT_ENDED_SCORE: i32 = 114; // 该活动已结束，您的成绩不会提交

// Account --------------------------------------------------------------------
#[allow(dead_code)]
const ACCOUNT_FROZEN_WARNING: i32 = 120; // 封号警告
#[allow(dead_code)]
const ACCOUNT_FROZEN: i32 = 121; // 账户冻结
#[allow(dead_code)]
const ACCOUNT_FROZEN_TEMP: i32 = 122; // 账户暂时冻结
#[allow(dead_code)]
const ACCOUNT_RESTRICTED: i32 = 123; // 账户被限制
#[allow(dead_code)]
const BLOCKED_IP_TEMP: i32 = 124; // 你今天不能再使用这个IP地址创建新的账号
#[allow(dead_code)]
const FUNCTION_USAGE_RESTRICTED: i32 = 150; // 非常抱歉您已被限制使用此功能
#[allow(dead_code)]
const FUNCTION_NOT_AVAILABLE: i32 = 151; // 目前无法使用此功能
#[allow(dead_code)]
const NO_USER_FOUND: i32 = 401; // 用户不存在
#[allow(dead_code)]
const CONNECT_FAILED: i32 = 403; // 无法连接至服务器

// Serial Number --------------------------------------------------------------
#[allow(dead_code)]
const GET_ITEM_FAILED: i32 = 501; // 此物品目前无法获取
                                    // const GET_ITEM_FAILED: i32 = 502; // 此物品目前无法获取
#[allow(dead_code)]
const INVALID_SERIAL_NUMBER: i32 = 504; // 无效的序列码
#[allow(dead_code)]
const SERIAL_NUMBER_ALREADY_USED: i32 = 505; // 此序列码已被使用
#[allow(dead_code)]
const ITEM_ALREADY_ACQUIRED: i32 = 506; // 你已拥有了此物品

// Friend list ----------------------------------------------------------------
#[allow(dead_code)]
const FRIEND_LIST_FULL: i32 = 601; // 好友列表已满
#[allow(dead_code)]
const ALREADY_FRIEND: i32 = 602; // 此用户已是好友
#[allow(dead_code)]
const SELF_FRIEND: i32 = 604; // 你不能加自己为好友
#[allow(dead_code)]

// Download -------------------------------------------------------------------
const DOWNLOAD_LIMIT_MEETS: i32 = 903; // 下载量超过了限制，请24小时后重试
#[allow(dead_code)]
const WAIT_24H: i32 = 905; // 请在再次使用此功能前等待24小时
#[allow(dead_code)]
const DEVICE_COUNT_LIMIT_MEETS: i32 = 1001; // 设备数量达到上限
#[allow(dead_code)]
const DEVICE_ALREAY_USED_THIS_FUNCTION: i32 = 1002; // 此设备已使用过此功能
#[allow(dead_code)]
const ERROR_DURING_DOWNLOADING: i32 = 9801; // 下载歌曲时发生问题，请再试一次
#[allow(dead_code)]
const DEVICE_STORAGE_FULL_ERROR: i32 = 9802; // 保存歌曲时发生问题，请检查设备空间容量

// Data Backup ----------------------------------------------------------------
#[allow(dead_code)]
const NO_DATA_ON_CLOUD: i32 = 9905; // 没有在云端发现任何数据
#[allow(dead_code)]
const ERROR_DURING_UPDATING_DATA: i32 = 9907; // 更新数据时发生了问题
                                                // const VERSION_TOO_OLD: i32 = 9908; // 服务器只支持最新的版本，请更新Arcaea

pub async fn handle_rejection(
    err: warp::Rejection,
) -> std::result::Result<impl warp::Reply, Infallible> {
    let status: StatusCode;
    let message: &str;
    let error_code: i32;

    if err.is_not_found() {
        status = StatusCode::NOT_FOUND;
        message = "not found";
        error_code = UNKNOWN_ERROR;
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        status = StatusCode::METHOD_NOT_ALLOWED;
        message = "method not allowed";
        error_code = UNKNOWN_ERROR;
    } else if let Some(e) = err.find::<ZrcDBError>() {
        match e {
            ZrcDBError::DataNotFound(msg) => {
                status = StatusCode::NOT_FOUND;
                message = "data needed not found";
                error_code = UNKNOWN_ERROR;
                log::error!("data not found, {}", msg);
            }
            ZrcDBError::Internal(msg, error) => {
                status = StatusCode::INTERNAL_SERVER_ERROR;
                message = "server side error";
                error_code = UNKNOWN_ERROR;
                log::error!("data access internal error, {}, {}", msg, error);
            }
            ZrcDBError::Other(msg) => {
                status = StatusCode::INTERNAL_SERVER_ERROR;
                message = "unknown database error";
                error_code = UNKNOWN_ERROR;
                log::error!("other data access error, {}", msg);
            }
        }
    } else {
        status = StatusCode::INTERNAL_SERVER_ERROR;
        message = "internal server error";
        error_code = UNKNOWN_ERROR;
        log::error!("unhandled error, {:?}", err);
    }

    let json = warp::reply::json(&ResponseContainer {
        success: false,
        value: (),
        error_code,
        error_msg: message.into(),
    });

    Ok(warp::reply::with_status(json, status))
}
