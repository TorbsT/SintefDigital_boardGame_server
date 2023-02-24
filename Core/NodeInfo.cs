using System.Collections.Generic;

namespace Core;

public struct NodeInfo
{
    public int ID;
    public List<NodeInfo> Neighbours;
}