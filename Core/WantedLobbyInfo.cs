namespace Core;

public struct WantedLobbyInfo
{
    public PlayerInfo PlayerInfo { get; set; }
    public string LobbyName { get; set; }

    public static implicit operator WantedLobbyInfo((PlayerInfo, string) tuple)
    {
        return new WantedLobbyInfo() {PlayerInfo = tuple.Item1, LobbyName = tuple.Item2};
    }

    public static implicit operator (PlayerInfo, string)(WantedLobbyInfo lobbyInfo)
    {
        return (lobbyInfo.PlayerInfo, lobbyInfo.LobbyName);
    }
}