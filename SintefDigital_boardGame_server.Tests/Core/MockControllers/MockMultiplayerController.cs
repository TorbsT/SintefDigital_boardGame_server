using SintefDigital_boardGame_server.Core;

namespace SintefDigital_boardGame_server.Test.Core.MockControllers;

public class MockMultiplayerController : IMultiplayerGameController
{
    private readonly MockMultiplayerPlayerInput _mockMultiplayerPlayerInput;
    
    public MockMultiplayerController(MockMultiplayerPlayerInput mockPlayerInput)
    {
        _mockMultiplayerPlayerInput = mockPlayerInput;
    }
    
    public void SendNewGameStateToPlayers(GameState state)
    {
        _mockMultiplayerPlayerInput.NotifyNewGameState(state);
    }
}