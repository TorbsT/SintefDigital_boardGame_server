using System.Collections.Generic;

namespace Core;

public struct NodeInfo
{
    public int ID;
    public string Name;

    public NodeInfo(int id, string name)
    {
        this.ID = id;
        this.Name = name;
    }
}