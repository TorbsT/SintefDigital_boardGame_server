namespace SintefDigital_boardGame_server.Core;

public struct Player
{
    public int ID;
    public string Name;
    public GameState ConnectedGame;
    public Node Position;
}