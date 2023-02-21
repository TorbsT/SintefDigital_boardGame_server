// The following piece of code is copied from TorbuTils
// https://github.com/TorbsT/Torbutils/blob/main/Packages/com.torbst.torbutils/Runtime/Giraphe/Graph.cs

using System;
using System.Collections.Generic;

namespace SintefDigital_boardGame_server.Helpers
{
    /// <summary>
    /// Representation of a graph (computer science)
    /// </summary>
    [Serializable]
    public class Graph
    {
        /// <summary>
        /// Quantity of nodes that have ever been mentioned in edges or satellite data.
        /// </summary>
        public int NodeCount => Nodes.Count;
        public string? Name { get; set; }
        private Edgees Edges { get; set; } = new();
        private Edgees AntiEdges { get; set; } = new();  // improves time complexity of CopyEdgesTo
        private Dictionary<(int, int), int> Weights { get; set; } = new();
        private Dictionary<int, Dictionary<string, object>> NodeInfo { get; set; } = new();
        private HashSet<int> Nodes { get; set; } = new();

        /// <summary>
        /// Creates a new Graph object with a
        /// copy of inputGraph's satellite data
        /// </summary>
        /// <param name="inputGraph"></param>
        /// <returns></returns>
        public static Graph MakeFromNodeInfo(Graph inputGraph)
        {
            Graph result = new();
            for (int i = 0; i < inputGraph.NodeCount; i++)
            {
                if (!inputGraph.NodeInfo.ContainsKey(i)) continue;
                foreach (string key in inputGraph.NodeInfo[i].Keys)
                {
                    object value = inputGraph.NodeInfo[i][key];
                    result.SetNodeInfo(i, key, value);
                }
            }
            return result;
        }

        /// <summary>
        /// Gets a copy of every node in this graph.
        /// A node is defined either though edges or satellite data.
        /// </summary>
        /// <returns>A collection of node IDs</returns>
        public ICollection<int> CopyNodeIds()
        {
            HashSet<int> result = new();
            foreach (int id in Nodes)
            {
                result.Add(id);
            }
            return result;
        }
        /// <summary>
        /// Gets a copy of every edge in this graph.
        /// </summary>
        /// <returns>
        /// A collection of tuples (from, to)
        /// </returns>
        public ICollection<(int, int)> CopyEdges()
        {
            HashSet<(int, int)> edges = new();
            foreach (int from in Edges.GetNodes())
            {
                foreach (int to in Edges.GetNodeEdges(from))
                {
                    edges.Add((from, to));
                }
            }
            return edges;
        }
        /// <summary>
        /// Gets a copy of every node directly accessible from node
        /// </summary>
        /// <returns>
        /// A collection of node ids, empty if node is nonexistent
        /// </returns>
        public ICollection<int> CopyEdgesFrom(int nodeId)
        {
            if (!Edges.HasNode(nodeId)) return new HashSet<int>();

            HashSet<int> result = new();
            foreach (int i in Edges.GetNodeEdges(nodeId))
            {
                result.Add(i);
            }

            return result;
        }
        /// <summary>
        /// Gets a copy of every node with direct access to node
        /// </summary>
        /// <returns>
        /// A collection of node ids, empty if node is nonexistent
        /// </returns>
        public ICollection<int> CopyEdgesTo(int nodeId)
        {
            if (!AntiEdges.HasNode(nodeId)) return new HashSet<int>();

            HashSet<int> result = new();
            foreach (int i in AntiEdges.GetNodeEdges(nodeId))
            {
                result.Add(i);
            }

            return result;
        }
        /// <summary>
        /// Gets the quantity of nodes directly accessible from node
        /// </summary>
        /// <returns>An integer, 0 if node is nonexistent</returns>
        public int GetEdgesCountFrom(int nodeId)
        {
            if (!Edges.HasNode(nodeId)) return 0;
            return Edges.GetNodeEdges(nodeId).Count;
        }
        /// <summary>
        /// Gets the quantity of nodes with direct access to node
        /// </summary>
        /// <returns>An integer, 0 if node is nonexistent</returns>
        public int GetEdgesCountTo(int nodeId)
        {
            if (!AntiEdges.HasNode(nodeId)) return 0;
            return AntiEdges.GetNodeEdges(nodeId).Count;
        }
        /// <summary>
        /// Adds a one-directional edge to this graph.
        /// Can be weighted.
        /// Replaces an existing edge if necessary.
        /// </summary>
        /// <param name="fromNodeId">Edge start node id.</param>
        /// <param name="toNodeId">Edge end node id.</param>
        /// <param name="weight">Edge weight. Ignore this parameter in unweighted graphs.</param>
        public void BuildEdge(int fromNodeId, int toNodeId, int weight = 1)
        {
            Edges.Connect(fromNodeId, toNodeId);
            AntiEdges.Connect(toNodeId, fromNodeId);
            SetWeight(fromNodeId, toNodeId, weight);
            if (!Nodes.Contains(fromNodeId)) Nodes.Add(fromNodeId);
            if (!Nodes.Contains(toNodeId)) Nodes.Add(toNodeId);
        }
        /// <summary>
        /// Removes the given edge from this graph.
        /// Only removes in the given direction.
        /// If the given edge is nonexistent, nothing happens.
        /// </summary>
        /// <param name="fromNodeId">Edge start node id.</param>
        /// <param name="toNodeId">Edge end node id.</param>
        public void RemoveEdge(int fromNodeId, int toNodeId)
        {
            Edges.Disconnect(fromNodeId, toNodeId);
            AntiEdges.Disconnect(toNodeId, fromNodeId);
        }
        /// <summary>
        /// Gets satellite info of a node.
        /// Can get from nonexistent nodes.
        /// </summary>
        /// <param name="nodeId">The node id.</param>
        /// <param name="infoKey">Specifies where the info is stored.</param>
        /// <returns>
        /// An object, null if there is no satellite info
        /// </returns>
        public object? GetNodeInfo(int nodeId, string infoKey)
        {
            if (!NodeInfo.ContainsKey(nodeId)) return null;
            if (!NodeInfo[nodeId].ContainsKey(infoKey)) return null;
            return NodeInfo[nodeId][infoKey];
        }
        /// <summary>
        /// Stores satellite info on a node.
        /// Can store on nonexistent nodes.
        /// Overwrites previous info at the given satellite
        /// </summary>
        /// <param name="nodeId">The node id.</param>
        /// <param name="infoKey">Specifies where the info should be stored.</param>
        /// <param name="infoValue">Specifies the object to store</param>
        public void SetNodeInfo(int nodeId, string infoKey, object infoValue)
        {
            if (!NodeInfo.ContainsKey(nodeId)) NodeInfo[nodeId] = new();
            if (!Nodes.Contains(nodeId)) Nodes.Add(nodeId);
            NodeInfo[nodeId][infoKey] = infoValue;
        }
        /// <summary>
        /// Sets the weight of an edge.
        /// Can set the weight of a nonexistent edge.
        /// </summary>
        /// <param name="fromNodeId"></param>
        /// <param name="toNodeId"></param>
        /// <param name="weight"></param>
        public void SetWeight(int fromNodeId, int toNodeId, int weight)
        {
            Weights[(fromNodeId, toNodeId)] = weight;
        }
        /// <summary>
        /// Gets the weight of an edge.
        /// </summary>
        /// <param name="fromNodeId">Edge start node id.</param>
        /// <param name="toNodeId">Edge end node id.</param>
        /// <returns>An integer, null if the edge weight is nonexistent.</returns>
        public int? GetWeight(int fromNodeId, int toNodeId)
        {
            if (!Weights.ContainsKey((fromNodeId, toNodeId))) return null;
            return Weights[(fromNodeId, toNodeId)];
        }
        /// <summary>
        /// Gets the quantity of edges between two nodes.
        /// </summary>
        /// <param name="nodeIdA">Node id A.</param>
        /// <param name="nodeIdB">Node id B.</param>
        /// <returns>0, 1 or 2.</returns>
        public int GetEdgeQuantityBetween(int nodeIdA, int nodeIdB)
        {
            int quantity = 0;
            if (Edges.HasEdge(nodeIdA, nodeIdB)) quantity++;
            if (Edges.HasEdge(nodeIdB, nodeIdA)) quantity++;
            return quantity;
        }

        /// <summary>
        /// "Edges" was taken.
        /// </summary>
        private class Edgees
        {
            private readonly Dictionary<int, HashSet<int>> edges = new();
            internal int NodeCount { get; private set; }

            internal bool HasNode(int id) => edges.ContainsKey(id);
            internal bool HasEdge(int from, int to)
                => edges.ContainsKey(from) && edges[from].Contains(to);
            internal ICollection<int> GetNodes() => edges.Keys;
            internal ICollection<int> GetNodeEdges(int id) => edges[id];
            internal void Connect(int from, int to)
            {
                if (!edges.ContainsKey(from))
                {
                    edges[from] = new();
                    NodeCount++;
                }
                edges[from].Add(to);
            }
            internal void Disconnect(int from, int to)
            {
                if (edges.ContainsKey(from))
                {
                    edges[from].Remove(to);
                    if (edges[from].Count == 0)
                    {
                        edges.Remove(from);
                        NodeCount--;
                    }
                }
            }
        }
    }
}
