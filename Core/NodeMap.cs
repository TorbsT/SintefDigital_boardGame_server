using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Core
{
    public class NodeMap
    {
        //Maps every node and neighbour (as they are in the workshop version) and returns a list of Nodes with every neighbour preset
        public static List<Node> Map()
        {
            List<Node> nodeMap = new List<Node>();
            nodeMap.Add(new Node(new NodeInfo(0, "Factory")));
            nodeMap.Add(new Node(new NodeInfo(1, "Refinery")));
            nodeMap.Add(new Node(new NodeInfo(2, "Industry Park")));
            nodeMap.Add(new Node(new NodeInfo(3, "I1")));
            nodeMap.Add(new Node(new NodeInfo(4, "I2")));
            nodeMap.Add(new Node(new NodeInfo(5, "Port")));
            nodeMap.Add(new Node(new NodeInfo(6, "I3")));
            nodeMap.Add(new Node(new NodeInfo(7, "Beach")));
            nodeMap.Add(new Node(new NodeInfo(8, "Northside")));
            nodeMap.Add(new Node(new NodeInfo(9, "I4")));
            nodeMap.Add(new Node(new NodeInfo(10, "Central Station")));
            nodeMap.Add(new Node(new NodeInfo(11, "City Square")));
            nodeMap.Add(new Node(new NodeInfo(12, "Concert Hall")));
            nodeMap.Add(new Node(new NodeInfo(13, "Park & Ride")));
            nodeMap.Add(new Node(new NodeInfo(14, "East Town")));
            nodeMap.Add(new Node(new NodeInfo(15, "Food Court")));
            nodeMap.Add(new Node(new NodeInfo(16, "City Park")));
            nodeMap.Add(new Node(new NodeInfo(17, "Quarry")));
            nodeMap.Add(new Node(new NodeInfo(18, "I5")));
            nodeMap.Add(new Node(new NodeInfo(19, "I6")));
            nodeMap.Add(new Node(new NodeInfo(20, "I7")));
            nodeMap.Add(new Node(new NodeInfo(21, "I8")));
            nodeMap.Add(new Node(new NodeInfo(22, "West Town")));
            nodeMap.Add(new Node(new NodeInfo(23, "Lakeside")));
            nodeMap.Add(new Node(new NodeInfo(24, "Warehouses")));
            nodeMap.Add(new Node(new NodeInfo(25, "I9")));
            nodeMap.Add(new Node(new NodeInfo(26, "I10")));
            nodeMap.Add(new Node(new NodeInfo(27, "Terminal 1")));
            nodeMap.Add(new Node(new NodeInfo(28, "Terminal 2")));
            nodeMap.ElementAt(0).AddNeighbour(nodeMap.ElementAt(1));
            nodeMap.ElementAt(0).AddNeighbour(nodeMap.ElementAt(2));
            nodeMap.ElementAt(1).AddNeighbour(nodeMap.ElementAt(2));
            nodeMap.ElementAt(2).AddNeighbour(nodeMap.ElementAt(3));
            nodeMap.ElementAt(3).AddNeighbour(nodeMap.ElementAt(4));
            nodeMap.ElementAt(3).AddNeighbour(nodeMap.ElementAt(9));
            nodeMap.ElementAt(4).AddNeighbour(nodeMap.ElementAt(5));
            nodeMap.ElementAt(4).AddNeighbour(nodeMap.ElementAt(6));
            nodeMap.ElementAt(6).AddNeighbour(nodeMap.ElementAt(7));
            nodeMap.ElementAt(6).AddNeighbour(nodeMap.ElementAt(13));
            nodeMap.ElementAt(7).AddNeighbour(nodeMap.ElementAt(8));
            nodeMap.ElementAt(9).AddNeighbour(nodeMap.ElementAt(10));
            nodeMap.ElementAt(9).AddNeighbour(nodeMap.ElementAt(18));
            nodeMap.ElementAt(10).AddNeighbour(nodeMap.ElementAt(11));
            nodeMap.ElementAt(10).AddNeighbour(nodeMap.ElementAt(15));
            nodeMap.ElementAt(11).AddNeighbour(nodeMap.ElementAt(12));
            nodeMap.ElementAt(11).AddNeighbour(nodeMap.ElementAt(16));
            nodeMap.ElementAt(12).AddNeighbour(nodeMap.ElementAt(13));
            nodeMap.ElementAt(13).AddNeighbour(nodeMap.ElementAt(14));
            nodeMap.ElementAt(13).AddNeighbour(nodeMap.ElementAt(20));
            nodeMap.ElementAt(14).AddNeighbour(nodeMap.ElementAt(21));
            nodeMap.ElementAt(15).AddNeighbour(nodeMap.ElementAt(16));
            nodeMap.ElementAt(16).AddNeighbour(nodeMap.ElementAt(19));
            nodeMap.ElementAt(17).AddNeighbour(nodeMap.ElementAt(18));
            nodeMap.ElementAt(18).AddNeighbour(nodeMap.ElementAt(19));
            nodeMap.ElementAt(18).AddNeighbour(nodeMap.ElementAt(23));
            nodeMap.ElementAt(19).AddNeighbour(nodeMap.ElementAt(20));
            nodeMap.ElementAt(20).AddNeighbour(nodeMap.ElementAt(26));
            nodeMap.ElementAt(20).AddNeighbour(nodeMap.ElementAt(27));
            nodeMap.ElementAt(21).AddNeighbour(nodeMap.ElementAt(27));
            nodeMap.ElementAt(22).AddNeighbour(nodeMap.ElementAt(23));
            nodeMap.ElementAt(23).AddNeighbour(nodeMap.ElementAt(24));
            nodeMap.ElementAt(24).AddNeighbour(nodeMap.ElementAt(25));
            nodeMap.ElementAt(25).AddNeighbour(nodeMap.ElementAt(26));
            nodeMap.ElementAt(26).AddNeighbour(nodeMap.ElementAt(27));
            nodeMap.ElementAt(27).AddNeighbour(nodeMap.ElementAt(28));
            return nodeMap;
        }
    }
}
