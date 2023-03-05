using Logging;
using System;
using System.Collections.Generic;
using System.Security.Principal;
using System.Threading;


namespace Core;

public class GameController
{
    //TODO: handle unused unique IDs
    private readonly List<GameState> _games;
    private readonly ILogger _logger;
    private List<int> _uniqueIDs = new List<int>();

    public GameController(ILogger logger)
    {
        this._games = new List<GameState>();
        this._logger = logger;
    }

    public List<GameStateInfo> GetGameStateInfos()
    {
        lock (_games)
        {
            List<GameStateInfo> games = _games.ConvertAll(game => (GameStateInfo) game);
            return games;
        }
    }
    
    public int MakeNewPlayerID()
    {
        int newID = GenerateUnusedGameIDs(1).First();
        lock (_uniqueIDs) _uniqueIDs.Add(newID);
        lock (_uniqueIDs) _logger.Log(LogLevel.Debug, _uniqueIDs.Count.ToString());
        return newID;
    }

    private List<int> GenerateUnusedGameIDs(int amountOfNewIDs)
    {
        Random generator = new Random();
        List<int> newIDs = new List<int>();
        for (int i = 0; i < amountOfNewIDs; i++)
        {
            int id = generator.Next();

            for (int _ = 0; _ < 100_000; _++)
            {
                lock (_uniqueIDs) if (!_uniqueIDs.Contains(id) && !newIDs.Contains(id)) break;
                id = generator.Next();
            }
            newIDs.Add(id);
        }
        if (newIDs.Count < amountOfNewIDs) throw new Exception("Failed to generate the wanted amount of new unique IDs");
        return newIDs;
    }

    public GameStateInfo CreateNewGame(WantedLobbyInfo lobbyNameAndPlayerInfo)
    {
        var newGame = CreateNewGameAndAssignHost(lobbyNameAndPlayerInfo);
        lock(_games) _games.Add(newGame);
        return newGame;
    }

    private GameState CreateNewGameAndAssignHost(WantedLobbyInfo lobbyNameAndPlayerInfo)
    {
        _logger.Log(LogLevel.Debug, "Creating new game state.");
        lock (_uniqueIDs) if (!_uniqueIDs.Contains(lobbyNameAndPlayerInfo.PlayerInfo.UniqueID)) throw new Exception($"Player with unique ID {lobbyNameAndPlayerInfo.PlayerInfo.UniqueID} does not exist on the server side. Unable to create game. Unique ids: {string.Join(", ", _uniqueIDs)}");
        lock (_games) foreach (GameState gameState in _games) if (gameState.ContainsUniquePlayerID(lobbyNameAndPlayerInfo.PlayerInfo)) throw new ArgumentException($"Player with unique ID {lobbyNameAndPlayerInfo.PlayerInfo.UniqueID} is connected to a game in progress"); //TODO: This line is causing problems for the tests
        var newGame = new GameState(lobbyNameAndPlayerInfo.LobbyName, GenerateUnusedGameID());
        newGame.AssignPlayerToGame(lobbyNameAndPlayerInfo.PlayerInfo);
        _logger.Log(LogLevel.Debug, $"Done creating new Game State with ID {newGame.GetGameStateInfo().ID} and name {newGame.GetGameStateInfo().Name}.");
        return newGame;
    }

    private int GenerateUnusedGameID()
    {
        var randomGenerator = new Random();
        var ID = randomGenerator.Next();
        while (!IsGameIDUnique(ID))
        {
            ID = randomGenerator.Next();
        }
        return ID;
    }

    private bool IsGameIDUnique(int ID)
    {
        lock (_games) foreach (var game in _games)
        {
            if (game.GetGameStateInfo().ID == ID)
            {
                return false;
            }
        }
        return true;
    }

    public void HandlePlayerInput(Input input)
    {
        // TODO check if input is legal based on the game state once applicable
        _logger.Log(LogLevel.Debug, $"Handling inputs for PlayerInfo with uniqueID {input.PlayerInfo.UniqueID} and " +
                                                   $"name {input.PlayerInfo.Name} and input type {input.Type}.");
        switch (input.Type)
        {
            case PlayerInfoInputType.Movement:
                HandleMovement(input.PlayerInfo, input.RelatedNode);
                break;
            default:
                throw new ArgumentOutOfRangeException();
        }
        _logger.Log(LogLevel.Debug, $"Finished handling inputs for PlayerInfo with ID {input.PlayerInfo.UniqueID}.");
    }

    private void HandleMovement(PlayerInfo playerInfo, NodeInfo toNodeCopy)
    {
        try
        {
            lock(_games) {
                var game = _games.First(state => state.GetGameStateInfo().ID == playerInfo.ConnectedGameID);
                // TODO: Check here if the movement is valid once applicable and dont use toNodeCopy.
                playerInfo.Position = toNodeCopy;
                game.UpdatePlayersBasedOnInfos(new List<PlayerInfo>() {playerInfo});
            }
            _logger.Log(LogLevel.Debug, $"Moved player {playerInfo.InGameID} in {playerInfo.ConnectedGameID} to " +
                                        $"node with nodeID {toNodeCopy.ID}");
        }
        catch (InvalidOperationException e)
        {
            _logger.Log(LogLevel.Error, "Failed to move PlayerInfo because the game the PlayerInfo refers to " +
                                        $"doesn't exist or the PlayerInfo isn't in the game. " +
                                        $"GameID: {playerInfo.ConnectedGameID}. InGame PlayerInfoID {playerInfo.InGameID}. Error: {e}");
        }
        catch (Exception e)
        {
            _logger.Log(LogLevel.Error, $"Something went wrong when trying to move the PlayerInfo. Error {e}");
        }
    }

    public int GetAmountOfCreatedPlayerIDs()
    {
        int amount;
        lock (_uniqueIDs) amount = _uniqueIDs.Count;
        return amount;
    }
}