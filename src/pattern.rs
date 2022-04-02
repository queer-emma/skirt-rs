use std::{
    collections::{BTreeMap, HashMap},
    slice::Iter,
};

use nalgebra::{
    Scalar,
    Vector2,
    Vector3,
};
use rust_decimal::{
    prelude::{
        ToPrimitive,
        Zero,
    },
    Decimal,
};
use serde::{
    Deserialize,
    Serialize,
};
use svg::node::element::{
    path::Data,
    Path,
};

use crate::{
    aabb::{
        AsAABB,
        AABB,
    },
    error::Error,
    parameters::Parameters,
    render::{
        Render,
        Target,
    },
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CurvatureCoords {
    Relative,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Start,
    End,
    Both,
}

impl Direction {
    pub fn is_start(&self) -> bool {
        matches!(self, Direction::Start | Direction::Both)
    }

    pub fn is_end(&self) -> bool {
        matches!(self, Direction::End | Direction::Both)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    Length,
    Curve,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetProperties {
    // there's a lot more, but it's mostly about performance measurements
    pub templates: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Properties {
    pub curvature_coords: CurvatureCoords,
    pub normalize_panel_translation: bool,
    pub units_in_meter: Decimal,
    pub normalized_edge_loops: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Influence {
    pub edge_list: Vec<EdgeRef>,
    pub panel: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub influence: Vec<Influence>,
    pub range: [Decimal; 2],
    pub r#type: ParameterType,

    /// the default value
    pub value: Decimal,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    LengthEquality,
    CurveEquality,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constraint {
    pub influence: Vec<Influence>,
    pub r#type: ConstraintType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub pattern: Pattern,
    pub properties: Properties,

    pub parameters: BTreeMap<String, Parameter>,
    pub parameter_order: Vec<String>,

    pub constraints: HashMap<String, Constraint>,
    pub constraint_order: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("no such parameter: {name}")]
    NoSuchParameter { name: String },
    #[error("value {value} is out of range {} .. {}", .range[0], .range[1])]
    OutOfRange { value: Decimal, range: [Decimal; 2] },
    #[error("no such panel: {panel}")]
    NoSuchPanel { panel: String },
    #[error("no such edge: panel={panel}, {edge}")]
    NoSuchEdge { panel: String, edge: usize },
    #[error("no such vertex: {index}")]
    NoSuchVertex { index: usize },
}

impl Template {
    pub fn parameters<'a>(&'a self) -> OrderedIter<'a, Parameter> {
        OrderedIter {
            items: &self.parameters,
            order: self.parameter_order.iter(),
        }
    }

    pub fn with_parameters(&self, parameters: &Parameters) -> Result<Pattern, RenderError> {
        let mut pattern = self.pattern.clone();

        for (name, parameter) in &self.parameters {
            let parameter_value = parameters.parameters.get(name).ok_or_else(|| {
                RenderError::NoSuchParameter {
                    name: name.to_owned(),
                }
            })?;

            // see comments below
            if *parameter_value < parameter.range[0] || *parameter_value > parameter.range[1] {
                return Err(RenderError::OutOfRange {
                    value: *parameter_value,
                    range: parameter.range,
                });
            }

            //let new_value = update.apply(parameter.value);
            // i don't really know how we now this is additive. usually it's multiplicative
            // and you can kind of tell from the default value.
            assert_eq!(parameter.value.to_u64(), Some(1));

            // todo: check local constraints
            // the paper says that this is only the range we sample from, but that there is
            // also a parameter type.
            /*if new_value < parameter.range[0] || new_value > parameter.range[1] {
                return Err(RenderError::OutOfRange {
                    value: new_value,
                    range: parameter.range,
                });
            }*/

            // todo: apply to influences
            for influence in &parameter.influence {
                let panel = pattern.panels.get_mut(&influence.panel).ok_or_else(|| {
                    RenderError::NoSuchPanel {
                        panel: influence.panel.to_owned(),
                    }
                })?;

                for edge_ref in &influence.edge_list {
                    let edge = panel.edges.get(edge_ref.id).ok_or_else(|| {
                        RenderError::NoSuchEdge {
                            panel: influence.panel.to_owned(),
                            edge: edge_ref.id,
                        }
                    })?;

                    let _start = *panel.get_vertex(edge.endpoints[0])?;
                    let _end = *panel.get_vertex(edge.endpoints[1])?;

                    // change value
                    match parameter.r#type {
                        ParameterType::Length => {
                            // move vertices(?)
                            // todo: we might have to layout them ourselves after this and we might
                            // want to do this anyway. but we just need
                            // to preserve the orientation (grain line).
                            if edge_ref
                                .direction
                                .map(|direction| direction.is_start())
                                .unwrap_or_default()
                            {
                                //edge.endpoints[0]
                                // todo:
                                //  is_start: start = start - k * (end-start) ??
                                //  is_end:   end   = end + k * (end-start) ??
                            }
                            if edge_ref
                                .direction
                                .map(|direction| direction.is_end())
                                .unwrap_or_default()
                            {
                                // todo: move vertices
                            }
                        }
                        ParameterType::Curve => todo!(),
                    }
                }
            }
        }

        Ok(pattern)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StitchStrough {
    pub edge: usize,
    pub panel: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Stitch(Vec<StitchStrough>);

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    pub panels: BTreeMap<String, Panel>,
    pub panel_order: Vec<String>,
    pub stitches: Vec<Stitch>,
}

impl Pattern {
    pub fn panels<'a>(&'a self) -> OrderedIter<'a, Panel> {
        OrderedIter {
            items: &self.panels,
            order: self.panel_order.iter(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub translation: Vector3<Decimal>,
    pub rotation: Vector3<Decimal>,
    pub edges: Vec<Edge>,
    pub vertices: Vec<Vector2<Decimal>>,
}

impl Panel {
    pub fn get_vertex(&self, index: usize) -> Result<&Vector2<Decimal>, RenderError> {
        self.vertices
            .get(index)
            .ok_or_else(|| RenderError::NoSuchVertex { index })
    }

    pub fn get_vertex_mut(&mut self, index: usize) -> Result<&mut Vector2<Decimal>, RenderError> {
        self.vertices
            .get_mut(index)
            .ok_or_else(|| RenderError::NoSuchVertex { index })
    }
}

impl Render for Panel {
    type Context = ();

    fn render(&self, target: &mut Target, _context: &Self::Context) -> Result<(), Error> {
        for edge in &self.edges {
            let mut vertices = edge.endpoints.iter().map(|endpoint| {
                self.vertices
                    .get(*endpoint)
                    .unwrap_or_else(|| panic!("invalid vertex index: {}", endpoint))
            });

            let first = vertices
                .next()
                .unwrap_or_else(|| panic!("no vertices in edge"));
            let second = vertices
                .next()
                .unwrap_or_else(|| panic!("no vertices in edge"));

            let mut data =
                Data::new().move_to((first.x.to_f64().unwrap(), first.y.to_f64().unwrap()));

            if let Some(curvature) = &edge.curvature {
                // quadratic bezier curve
                data = data.smooth_quadratic_curve_to((
                    second.x.to_f64().unwrap(),
                    second.y.to_f64().unwrap(),
                    curvature[0].to_f64().unwrap(),
                    curvature[1].to_f64().unwrap(),
                ));
            }
            else {
                // straight line
                data = data.line_to((second.x.to_f64().unwrap(), second.y.to_f64().unwrap()));
            }

            let path = Path::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 3u64)
                .set("d", data);

            target.add(path);
        }

        todo!();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    /// endpoints indices. this references into the panel's vertex array.
    pub endpoints: [usize; 2],

    /// bezier curve control points. each `f32` is one control point? it only
    /// supports quadratic right now.
    ///
    /// > 2D coordinates of the quadratic Bezier curve control point (named
    /// curvature coordinates) > if the edge is not a straight line.
    #[serde(default)]
    pub curvature: Option<[Decimal; 2]>,
}

impl<T> AsAABB<T> for Edge
where
    T: Clone + Scalar + Zero + PartialOrd,
{
    fn as_aabb(&self) -> AABB<T> {
        todo!()
    }
}

pub struct OrderedIter<'a, T> {
    items: &'a BTreeMap<String, T>,
    order: Iter<'a, String>,
}

impl<'a, T> Iterator for OrderedIter<'a, T> {
    type Item = (&'a str, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let name = self.order.next()?;

        let item = self
            .items
            .get(name)
            .unwrap_or_else(|| panic!("item is in order, but doesn't exist: {}", name));

        Some((name.as_str(), item))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(from = "RawEdgeRef", into = "RawEdgeRef")]
pub struct EdgeRef {
    pub id: usize,
    pub direction: Option<Direction>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum RawEdgeRef {
    WithDirection { direction: Direction, id: usize },
    Id(usize),
}

impl From<RawEdgeRef> for EdgeRef {
    fn from(raw: RawEdgeRef) -> Self {
        match raw {
            RawEdgeRef::WithDirection { direction, id } => {
                Self {
                    id,
                    direction: Some(direction),
                }
            }
            RawEdgeRef::Id(id) => {
                Self {
                    id,
                    direction: None,
                }
            }
        }
    }
}

impl From<EdgeRef> for RawEdgeRef {
    fn from(edge_ref: EdgeRef) -> Self {
        if let Some(direction) = edge_ref.direction {
            Self::WithDirection {
                direction,
                id: edge_ref.id,
            }
        }
        else {
            Self::Id(edge_ref.id)
        }
    }
}
