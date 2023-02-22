using SintefDigital_boardGame_server.Core;
using SintefDigital_boardGame_server.Logging;
using SintefDigital_boardGame_server.Test.Core.MockControllers;
using Xunit;

namespace SintefDigital_boardGame_server.Test.Core;

public class GameStateTest
{
    [Fact]
    public void TestCreatingNewGame()
    {
        var mockMultiPlayerController = new MockMultiplayerController();
        var gameController = new GameController(new ThresholdLogger(LogLevel.Ignore, LogLevel.Ignore), mockMultiPlayerController, mockMultiPlayerController);
        
        // TODO: Write the tests!
        
    }
}