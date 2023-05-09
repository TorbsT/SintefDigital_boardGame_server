use std::{collections::HashMap, mem};

use crate::game_data::{enums::{district::District, restriction_type::RestrictionType}, custom_types::{NodeID, MovementCost}};

use super::{node::Node, neighbour_relationship::NeighbourRelationship, edge_restriction::EdgeRestriction, situation_card::SituationCard};

#[derive(Clone, Default, Debug)]
pub struct NodeMap {
    pub nodes: Vec<Node>,
    pub edges: HashMap<NodeID, Vec<NeighbourRelationship>>,
    pub neighbourhood_cost: HashMap<District, MovementCost>,
}

impl NodeMap {
    /// Creates a new empty NodeMap.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: HashMap::new(),
            neighbourhood_cost: HashMap::new(),
        }
    }

    /// Updates the district movement penalty of a district based on the situation card.
    pub fn update_neighbourhood_cost(&mut self, situation_card: &SituationCard) {
        for i in &situation_card.costs {
            self.neighbourhood_cost
                .insert(i.neighbourhood, i.traffic.get_movement_cost());
        }
    }

    /// Creates a new NodeMap with the default nodes and edges defined in the (7th) workshop version.
    /// 
    /// [`Self::new_default`]: #method.new_default
    #[must_use]
    pub fn new_default() -> Self {
        let mut map = Self::new();

        let node0: Node = Node::new(0, String::from("Factory"));
        let node1: Node = Node::new(1, String::from("Refinery"));
        let mut node2: Node = Node::new(2, String::from("Industry Park"));
        let node3: Node = Node::new(3, String::from("I1"));
        let node4: Node = Node::new(4, String::from("I2"));
        let node5: Node = Node::new(5, String::from("Port"));
        let node6: Node = Node::new(6, String::from("I3"));
        let node7: Node = Node::new(7, String::from("Beach"));
        let node8: Node = Node::new(8, String::from("Northside"));
        let mut node9: Node = Node::new(9, String::from("I4"));
        let mut node10: Node = Node::new(10, String::from("Central Station"));
        let node11: Node = Node::new(11, String::from("City Square"));
        let node12: Node = Node::new(12, String::from("Concert Hall"));
        let mut node13: Node = Node::new(13, String::from("Eastside Mart"));
        let node14: Node = Node::new(14, String::from("East Town"));
        let node15: Node = Node::new(15, String::from("Food Court"));
        let node16: Node = Node::new(16, String::from("City Park"));
        let node17: Node = Node::new(17, String::from("Quarry"));
        let node18: Node = Node::new(18, String::from("I5"));
        let mut node19: Node = Node::new(19, String::from("I6"));
        let node20: Node = Node::new(20, String::from("I7"));
        let mut node21: Node = Node::new(21, String::from("I8"));
        let node22: Node = Node::new(22, String::from("West Town"));
        let node23: Node = Node::new(23, String::from("Lakeside"));
        let mut node24: Node = Node::new(24, String::from("Warehouses"));
        let node25: Node = Node::new(25, String::from("I9"));
        let mut node26: Node = Node::new(26, String::from("I10"));
        let mut node27: Node = Node::new(27, String::from("Terminal 1"));
        let node28: Node = Node::new(28, String::from("Terminal 2"));

        node2.toggle_rail_connection();
        node10.toggle_rail_connection();
        node24.toggle_rail_connection();
        node27.toggle_rail_connection();

        node2.is_parking_spot = true;
        node9.is_parking_spot = true;
        node13.is_parking_spot = true;
        node19.is_parking_spot = true;
        node21.is_parking_spot = true;
        node26.is_parking_spot = true;
        node27.is_parking_spot = true;

        map.nodes.push(node0.clone());
        map.nodes.push(node1.clone());
        map.nodes.push(node2.clone());
        map.nodes.push(node3.clone());
        map.nodes.push(node4.clone());
        map.nodes.push(node5.clone());
        map.nodes.push(node6.clone());
        map.nodes.push(node7.clone());
        map.nodes.push(node8.clone());
        map.nodes.push(node9.clone());
        map.nodes.push(node10.clone());
        map.nodes.push(node11.clone());
        map.nodes.push(node12.clone());
        map.nodes.push(node13.clone());
        map.nodes.push(node14.clone());
        map.nodes.push(node15.clone());
        map.nodes.push(node16.clone());
        map.nodes.push(node17.clone());
        map.nodes.push(node18.clone());
        map.nodes.push(node19.clone());
        map.nodes.push(node20.clone());
        map.nodes.push(node21.clone());
        map.nodes.push(node22.clone());
        map.nodes.push(node23.clone());
        map.nodes.push(node24.clone());
        map.nodes.push(node25.clone());
        map.nodes.push(node26.clone());
        map.nodes.push(node27.clone());
        map.nodes.push(node28.clone());

        map.add_relationship(node0.clone(), node1.clone(), District::IndustryPark, 1, false);
        map.add_relationship(node0, node2.clone(), District::IndustryPark, 1, false);
        map.add_relationship(node1, node2.clone(), District::IndustryPark, 1, false);
        map.add_relationship(node2.clone(), node3.clone(), District::Suburbs, 1, false);
        map.add_relationship(node3.clone(), node4.clone(), District::RingRoad, 1, false);
        map.add_relationship(node3, node9.clone(), District::RingRoad, 1, false);
        map.add_relationship(node4.clone(), node5, District::Port, 1, false);
        map.add_relationship(node4, node6.clone(), District::RingRoad, 1, false);
        map.add_relationship(node6.clone(), node13.clone(), District::RingRoad, 1, false);
        map.add_relationship(node6, node7.clone(), District::Suburbs, 1, false);
        map.add_relationship(node7, node8, District::Suburbs, 1, false);
        map.add_relationship(node9.clone(), node10.clone(), District::CityCentre, 1, false);
        map.add_relationship(node9, node18.clone(), District::RingRoad, 1, false);
        map.add_relationship(node10.clone(), node11.clone(), District::CityCentre, 1, false);
        map.add_relationship(node10.clone(), node15.clone(), District::CityCentre, 1, false);
        map.add_relationship(node11.clone(), node12.clone(), District::CityCentre, 1, false);
        map.add_relationship(node11, node16.clone(), District::CityCentre, 1, false);
        map.add_relationship(node12, node13.clone(), District::CityCentre, 1, false);
        map.add_relationship(node13.clone(), node14.clone(), District::Suburbs, 1, false);
        map.add_relationship(node13, node20.clone(), District::RingRoad, 1, false);
        map.add_relationship(node14, node21.clone(), District::Suburbs, 1, false);
        map.add_relationship(node15, node16.clone(), District::CityCentre, 1, false);
        map.add_relationship(node16, node19.clone(), District::CityCentre, 1, false);
        map.add_relationship(node17, node18.clone(), District::Suburbs, 1, false);
        map.add_relationship(node18.clone(), node19.clone(), District::RingRoad, 1, false);
        map.add_relationship(node18, node23.clone(), District::Suburbs, 1, false);
        map.add_relationship(node19, node20.clone(), District::RingRoad, 1, false);
        map.add_relationship(node20.clone(), node26.clone(), District::Suburbs, 1, false);
        map.add_relationship(node20, node27.clone(), District::Airport, 1, false);
        map.add_relationship(node21, node27.clone(), District::Airport, 1, false);
        map.add_relationship(node22, node23.clone(), District::Suburbs, 1, false);
        map.add_relationship(node23, node24.clone(), District::Suburbs, 1, false);
        map.add_relationship(node24.clone(), node25.clone(), District::Suburbs, 1, false);
        map.add_relationship(node25, node26.clone(), District::Suburbs, 1, false);
        map.add_relationship(node26, node27.clone(), District::Airport, 1, false);
        map.add_relationship(node27.clone(), node28, District::Airport, 1, false);

        map.add_relationship(node2, node10.clone(), District::IndustryPark, 1, true);
        map.add_relationship(node10, node24.clone(), District::IndustryPark, 1, true);
        map.add_relationship(node24, node27, District::IndustryPark, 1, true);

        let mut neighbourhood = District::first();
        map.change_neighbourhood_cost(neighbourhood, 1);
        while let Some(n) = neighbourhood.next() {
            neighbourhood = n;
            map.change_neighbourhood_cost(n, 1);
        }

        map
    }

    /// Replaces self with the default map defined in [`Self::new_default`].
    pub fn reset(&mut self) {
        let _ = mem::replace(self, Self::new_default());
    }

    /// Toggles the `is_connected_to_rail` bool of the node with the given ID.
    pub fn toggle_rail_connection_on_node_with_id(&mut self, node_id: NodeID) -> Result<(), String> {
        let Some(node) = self.nodes.iter_mut().find(|node| node.id == node_id) else {
            return Err(format!("There is no node with the given ID: {}", node_id));
        };
        node.toggle_rail_connection();
        Ok(())
    }

    /// Gets the node with the given ID. Returns an error if there is no node with the given ID.
    pub fn get_node_by_id(&self, position_node_id: NodeID) -> Result<Node, String> {
        self.nodes
            .iter()
            .find(|&node| node.id == position_node_id)
            .map_or_else(
                || {
                    Err(format!(
                        "There is no node with the given ID: {}",
                        position_node_id
                    ))
                },
                |node| Ok(node.clone()),
            )
    }

    /// Gets all the neighbouring edges of the node with the given ID. Returns none if there are no edges for the given node.
    pub fn get_neighbour_relationships_of_node_with_id(
        &self,
        node_id: NodeID,
    ) -> Option<Vec<NeighbourRelationship>> {
        self.edges.get(&node_id).cloned()
    }

    /// Changes the district cost of the given neighbourhood.
    pub fn change_neighbourhood_cost(&mut self, neighbourhood: District, cost: MovementCost) {
        self.neighbourhood_cost.insert(neighbourhood, cost);
    }

    /// Get's the cost of moving within the district (not counting moving along the edge itself). Returns an error if something went wrong.
    pub fn first_time_in_district_cost(
        &self,
        neighbour_relationship: NeighbourRelationship,
    ) -> Result<MovementCost, String> {
        let Some(neighbourhood_cost) = self.neighbourhood_cost.get(&neighbour_relationship.neighbourhood) else {
            return Err(format!("There was no neighbourhood_cost in the nodemap for neighbourhood {:?}", neighbour_relationship.neighbourhood));
        };
        Ok(*neighbourhood_cost)
    }

    /// Checks if the given node IDs are neighbours. Returns an error if something went wrong.
    pub fn are_nodes_neighbours(&self, node_1: NodeID, node_2: NodeID) -> Result<bool, String> {
        let Some(neighbours) = self.edges.get(&node_1) else {
            return Err(format!("There is no node with id {} that has any neighbour with id {}!", node_1, node_2));
        };
        Ok(neighbours
            .iter()
            .any(|relationship| relationship.to == node_2))
    }

    fn add_relationship(
        &mut self,
        node1: Node,
        node2: Node,
        neighbourhood: District,
        cost: MovementCost,
        is_connected_through_rail: bool,
    ) {
        let mut relationship = NeighbourRelationship::new(node2.id, neighbourhood, cost, is_connected_through_rail);
        self.edges
            .entry(node1.id)
            .or_default()
            .push(relationship.clone());
        relationship.to = node1.id;
        self.edges.entry(node2.id).or_default().push(relationship);
    }

    /// Adds the given edge restriction to the map and if the edge restriction is modifiable (removable), and returns an error if something went wrong.
    pub fn set_restriction_on_edge(
        &mut self,
        edge_restriction: &EdgeRestriction,
        modifiable: bool,
    ) -> Result<(), String> {
        match self.set_restriction_on_relationship(edge_restriction.node_one, edge_restriction.node_two, edge_restriction.edge_restriction, modifiable) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        if edge_restriction.edge_restriction == RestrictionType::OneWay {
            return Ok(()); // If the restriction is one way, we don't need to set the other way
        }
        match self.set_restriction_on_relationship(edge_restriction.node_two, edge_restriction.node_one, edge_restriction.edge_restriction, modifiable) {
            Ok(_) => Ok(()),
            Err(e) => {
                let mut err_string = String::new();
                match self.remove_restriction_from_relationship(edge_restriction.node_one, edge_restriction.node_two) {
                    Ok(_) => (),
                    Err(e) => err_string = e,
                }
                Err(format!("{} and secondly {}", e, err_string))
            }
        }
    }

    fn set_restriction_on_relationship(
        &mut self,
        from_node_id: NodeID,
        to_node_id: NodeID,
        restriction_type: RestrictionType,
        modifiable: bool,
    ) -> Result<(), String> {
        match self.are_nodes_neighbours(from_node_id, to_node_id) {
            Ok(n) => {
                if !n {
                    return Err(format!("The node {} is not neighbours with node {} and can therefore not put park and ride between them!", from_node_id, to_node_id));
                }
            }
            Err(e) => return Err(e),
        }
        let Some(neighbours) = self.edges.get_mut(&from_node_id) else {
            return Err(format!("There is no node with id {} that has any neighbours! Therefore we cannot place park and ride!", from_node_id));
        };

        for mut neighbour in neighbours {
            if neighbour.to != to_node_id {
                continue;
            }
            neighbour.restriction = Some(restriction_type);
            neighbour.is_modifiable = modifiable;
        }
        Ok(())
    }

    fn remove_restriction_from_relationship(
        &mut self,
        from_node_id: NodeID,
        to_node_id: NodeID,
    ) -> Result<(), String> {
        match self.are_nodes_neighbours(from_node_id, to_node_id) {
            Ok(n) => {
                if !n {
                    return Err(format!("The node {} is not neighbours with node {} and can therefore not put park and ride between them!", from_node_id, to_node_id));
                }
            }
            Err(e) => return Err(e),
        }
        let Some(neighbours) = self.edges.get_mut(&from_node_id) else {
            return Err(format!("There is no node with id {} that has any neighbours! Therefore we cannot place park and ride!", from_node_id));
        };

        for mut neighbour in neighbours {
            if neighbour.to != to_node_id {
                continue;
            }
            if !neighbour.is_modifiable {
                return Err(format!("The edge between node {} and node {} is not modifiable!", from_node_id, to_node_id));
            }
            neighbour.restriction = None;
        }
        Ok(())
    }

    /// Tries to remove the given edge restriction from the map and returns an error if something went wrong.
    pub fn remove_restriction_from_edge(
        &mut self,
        edge_restriction: &EdgeRestriction,
    ) -> Result<(), String> {
        match self.remove_restriction_from_relationship(edge_restriction.node_one, edge_restriction.node_two) {
            Ok(_) => (),
            Err(e) => return Err(e),
        }
        match self.remove_restriction_from_relationship(edge_restriction.node_two, edge_restriction.node_one) {
            Ok(_) => Ok(()),
            Err(e) => {
                let mut err_string = String::new();
                match self.set_restriction_on_edge(edge_restriction, true) {
                    Ok(_) => (),
                    Err(e) => err_string = e,
                }
                Err(format!("{} and secondly {}", e, err_string))
            }
        }
    }
}