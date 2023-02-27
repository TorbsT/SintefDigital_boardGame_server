using System;
using System.IO;

using System.Runtime.CompilerServices;

namespace Logging;

/// <summary>
/// This class is theoretically thread safe
/// </summary>
public class ThresholdLogger : ILogger
{
    private readonly LogLevel _printThreshold;
    private readonly LogLevel _storeThreshold;
    private uint _fileIndex = 1;
    private ReaderWriterLockSlim _rwLock = new ReaderWriterLockSlim();

    public ThresholdLogger() : this(LogLevel.Info, LogLevel.Info)
    {
        
    }
    
    public ThresholdLogger(LogLevel printThreshold, LogLevel storeThreshold)
    {
        this._printThreshold = printThreshold;
        this._storeThreshold = storeThreshold;
    }
    
    // TODO: callingClass here can be show sensitive information (filepaths), find out if this is a problem or not
    public void Log(LogLevel severityLevel, string logData, [CallerMemberName] string callingFunction = "", [CallerFilePath] string callingClass = "")
    {
        _rwLock.EnterWriteLock();
        try
        {
            HandleLogPrint(severityLevel, logData, callingClass, callingFunction);
            HandleStoringOfLog(severityLevel, logData, callingClass, callingFunction);
        }
        catch (Exception e)
        {
            Console.WriteLine($"Something failed asynchronously when logging. Error: {e}");
        }
        finally
        {
            _rwLock.ExitWriteLock();
        }
    }
    private void HandleLogPrint(LogLevel severityLevel, string logData, string callingClass, string callingFunction)
    {
        if (_printThreshold == LogLevel.Ignore || severityLevel < _printThreshold) return;
        
        Console.WriteLine(CreateLoggingMessage(severityLevel, logData, callingClass, callingFunction));
    }

    private string CreateLoggingMessage(LogLevel severityLevel, string logData, string callingClass, string callingFunction)
    {
        int lastBackslashPosition = callingClass.LastIndexOf("\\", StringComparison.Ordinal) + 1;
        string className = callingClass.Substring(lastBackslashPosition, callingClass.Length - lastBackslashPosition);
        return $"{DateTime.Now} [{severityLevel}] in {callingFunction} in {className} | {logData}";
    }
    
    private void HandleStoringOfLog(LogLevel severityLevel, string logData, string callingClass, string callingFunction)
    {
        if (_storeThreshold == LogLevel.Ignore || severityLevel < _storeThreshold) return;

        var filePath = CreateFilePath();

        try
        {
            StreamWriter writer = new StreamWriter(filePath, true);
            writer.WriteLine(CreateLoggingMessage(severityLevel, logData, callingClass, callingFunction));
            writer.Dispose();
        }
        catch (Exception e)
        {
            Console.WriteLine(CreateLoggingMessage(LogLevel.Error, $"Failed to store data to file. Error: {e}. Data" + CreateLoggingMessage(severityLevel, logData, callingClass, callingFunction),  callingClass, callingFunction));
        }
    }

    private string CreateFilePath()
    {
        var fileName = CreateFileName();
        var filePath = CreateFilePathForFileName(fileName);

        do
        {
            if (new FileInfo(filePath).Length < LoggingConstants.MaxFileSize) break;
            _fileIndex++;
            fileName = CreateFileName();
            filePath = CreateFilePathForFileName(fileName);
        } while (File.Exists(filePath));

        return filePath;
    }

    private string CreateFileName()
    {
        return $"threshold_logger_{DateTime.Now.Date:d}_{_fileIndex}.txt";
    }

    private string CreateFilePathForFileName(string fileName)
    {
        var folderPath = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, LoggingConstants.FolderName);
        return Path.Combine(folderPath, fileName);
    }
    
}