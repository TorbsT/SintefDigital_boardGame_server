namespace SintefDigital_boardGame_server.Logging;

public static class LoggingConstants
{
    public const string FolderName = "BoardGameServerLogs";
    public const uint MaxFileSize = 256 * 1024 * 1024;
}

public interface ILogger
{
    void Log(LogLevel severityLevel, string logData);
}