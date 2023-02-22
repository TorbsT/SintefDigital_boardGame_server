namespace SintefDigital_boardGame_server.Core;

public struct Player
{
    public int inGameID;
    public int uniqueID;
    public string Name;
    public GameState ConnectedGame;
    public Node Position;
}