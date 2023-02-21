using System.Collections.Generic;

namespace SintefDigital_boardGame_server.Core;

public struct Node
{
    public int ID;
    public List<Node> Neighbours;
}