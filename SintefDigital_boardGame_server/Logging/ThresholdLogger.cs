using System;
using System.IO;

namespace SintefDigital_boardGame_server.Logging;

public class ThresholdLogger : ILogger
{
    private readonly LogLevel _printThreshold;
    private readonly LogLevel _storeThreshold;
    private uint _fileIndex = 1;


    public ThresholdLogger() : this(LogLevel.Info, LogLevel.Info)
    {
        
    }
    
    public ThresholdLogger(LogLevel printThreshold, LogLevel storeThreshold)
    {
        this._printThreshold = printThreshold;
        this._storeThreshold = storeThreshold;
    }
    
    public void Log(LogLevel severityLevel, string logData)
    {
        HandleLogPrint(severityLevel, logData);
        HandleStoringOfLog(severityLevel, logData);
    }
    private void HandleLogPrint(LogLevel severityLevel, string logData)
    {
        if (_printThreshold == LogLevel.Ignore || severityLevel < _printThreshold) return;
        
        Console.WriteLine(CreateLoggingMessage(severityLevel, logData));
    }

    private string CreateLoggingMessage(LogLevel severityLevel, string logData)
    {
        return $"{DateTime.Now} [{severityLevel}] {logData}";
    }
    
    private void HandleStoringOfLog(LogLevel severityLevel, string logData)
    {
        if (_storeThreshold == LogLevel.Ignore || severityLevel < _storeThreshold) return;

        var filePath = CreateFilePath();

        try
        {
            StreamWriter writer = new StreamWriter(filePath, true);
            writer.WriteLine(CreateLoggingMessage(severityLevel, logData));
            writer.Dispose();
        }
        catch (Exception e)
        {
            Console.WriteLine(CreateLoggingMessage(LogLevel.Error, "Failed to store data to file. Data" + CreateLoggingMessage(severityLevel, logData)));
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