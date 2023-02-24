using System.Runtime.CompilerServices;

namespace Logging;

public static class LoggingConstants
{
    public const string FolderName = "BoardGameServerLogs";
    public const uint MaxFileSize = 256 * 1024 * 1024;
}

public interface ILogger
{
    void Log(LogLevel severityLevel, string logData, [CallerMemberName] string callingFunction = "", [CallerFilePath] string callingClass = "");
}