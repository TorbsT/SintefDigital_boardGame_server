namespace SintefDigital_boardGame_server.Core;

public class RwLock<T> : IDisposable
{
    private readonly ReaderWriterLockSlim _lock = new ReaderWriterLockSlim();
    private T _obj;

    public RwLock(T obj)
    {
        _obj = obj;
    }

    public T Lock()
    {
        _lock.EnterWriteLock();
        return _obj;
    }

    public void ReleaseLock()
    {
        _lock.ExitReadLock();
    }


    public void Dispose()
    {
        _lock.Dispose();
    }
}