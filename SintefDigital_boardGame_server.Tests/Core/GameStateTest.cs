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
        RwLock<IMultiplayerPlayerInputController> inputController;
        RwLock<IMultiplayerViewController> viewController;
        {
            var mockMultiPlayerController = new MockMultiplayerController();
            inputController = new RwLock<IMultiplayerPlayerInputController>(mockMultiPlayerController);
            viewController = new RwLock<IMultiplayerViewController>(mockMultiPlayerController);
        }
        var gameController = new GameController(new ThresholdLogger(LogLevel.Ignore, LogLevel.Ignore), viewController, inputController);
        
        // TODO: Write the tests!
    }
}