using System.Collections.Generic;

namespace Core;

public struct NodeInfo
{
    public int ID { get; set; }
    public string? Name { get; set; }

    public NodeInfo(int id, string name)
    {
        this.ID = id;
        this.Name = name;
    }
}