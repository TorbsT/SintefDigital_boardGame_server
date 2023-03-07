
pub mod logger;
pub mod threshold_logger;

const LOG_FOLDER_NAME: &'static str = "BoardGameServerLogs";
const MAX_FILE_SIZE: u64 = 256 * 1024 * 1024;

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
