namespace Core;

public struct PlayerInfo
{
    public int ConnectedGameID  { get; set; }
    public int InGameID  { get; set; }
    public int UniqueID  { get; set; }
    public string Name  { get; set; }
    public NodeInfo Position  { get; set; }
}