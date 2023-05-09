///! This library contains a logger trait and a threshold logger struct that implements the logger trait.

/// The logger module contains a logger trait and the enum/datastructures it uses.
pub mod logger;
/// The threshold_logger module contains a threshold logger struct that implements the logger trait.
pub mod threshold_logger;

const LOG_FOLDER_NAME: &str = "BoardGameServerLogs";
/// The maximum size of a log file in bytes.
const MAX_FILE_SIZE: u64 = 256 * 1024 * 1024;