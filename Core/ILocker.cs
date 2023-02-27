namespace Core;

/// <summary>
/// It's important to make sure that each public function checks if the lock is activated before making any changes.
/// It's also important to remember to lock the object itself in it's own functions if it changes any private
/// variables.
/// An example of how ILocker would be implemented:
///
/// private ReaderWriterLockSlim _lock = new ReaderWriterLockSlim();
/// 
/// public void Lock() {
///     _lock.EnterWriteLock();
/// }
/// public void ReleaseLock() {
///     _lock.ExitReadLock();
/// }
/// public void VerifyLock() {
///     if (!_lock.IsWriteLockHeld) throw new InvalidOperationException();
/// }
/// ...
/// public void SomeFunction() {
///     VerifyLock();
///     ...
/// }
/// </summary>
public interface ILocker
{
    public void Lock();
    public void ReleaseLock();
    public void VerifyLock();
}