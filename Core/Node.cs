using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Core;

// TODO: Make this internal once the graph and all the nodes are set correctly
public class Node
{
    private NodeInfo nodeInfo;
    private List<Node> neighbours;

    public Node(NodeInfo nodeInfoStruct = default)
    {
        if (EqualityComparer<NodeInfo>.Default.Equals(nodeInfoStruct, default))
        {
            this.nodeInfo = new NodeInfo();
        }
        else
        {
            this.nodeInfo = nodeInfoStruct;
        }
        this.neighbours = new List<Node>();
    }

    //Adds a neighbour node
    public void AddNeighbour(Node node)
    {
        this.neighbours.Add(node);
        node.GetNeighbours().Add(this);
    }

    //Returns a list of all neighbours
    public List<Node> GetNeighbours()
    {
        return this.neighbours;
    }

    //Returns nodeInfo struct
    public NodeInfo GetNodeInfo()
    {
        return this.nodeInfo;
    }

    public static implicit operator NodeInfo(Node node)
    {
        return node.nodeInfo;
    }
}
