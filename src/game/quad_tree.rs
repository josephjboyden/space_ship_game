use bevy::prelude::*;

use super::PLAYER_AREA_HALF_DIMENTION;

pub struct QuadTreePlugin;

impl Plugin for QuadTreePlugin {
    fn build(&self, app: &mut App) {
        app .insert_resource(QuadTree::new(AABB::new(Vec2::ZERO, PLAYER_AREA_HALF_DIMENTION)))
            .add_systems(PreUpdate, buildtree);
            //.add_systems(Update, render_tree);
    }
}

#[derive(Clone, Copy)]
pub struct AABB
{
    center: Vec2,
    half_dimention: f32,
}

impl AABB {
    pub fn new(center: Vec2, half_dimention: f32) -> Self {
        Self {
            center: center,
            half_dimention: half_dimention,
        }
    }

    fn contains_point(&self, point: Vec2) -> bool {
        !(
            point.x > self.center.x + self.half_dimention ||
            point.x < self.center.x - self.half_dimention ||
            point.y > self.center.y + self.half_dimention ||
            point.y < self.center.y - self.half_dimention
        )
    }

    fn inersects_aabb(&self, other: &AABB) -> bool {
        let dist = self.half_dimention + other.half_dimention;

        (self.center.x - other.center.x).abs() < dist &&
        (self.center.y - other.center.y).abs() < dist
    }
}

const CAPACITY: usize = 4;
#[derive(Resource)]
pub struct QuadTree {
    boundry: AABB,
    points: [Option<(Vec2, Entity)>; CAPACITY],
    i: usize,

    subtrees: Option<Box<(QuadTree, QuadTree, QuadTree, QuadTree)>>
}

impl QuadTree {
    fn new(boundry: AABB) -> Self {
        Self {
            boundry: boundry,
            points: [None; CAPACITY],
            i: 0,
            
            subtrees: None,
        }
    }

    fn insert(&mut self, point: Vec2, entity: Entity) -> bool{

        if !self.boundry.contains_point(point) {return false;}

        if self.i < CAPACITY && self.subtrees.is_none()
        {
            self.points[self.i] = Some((point, entity));
            self.i += 1;
            return true;
        }

        if self.subtrees.is_none() {
            self.subdivide();
        }

        match self.subtrees.as_mut(){
            Some(subtrees) => {
                if subtrees.0.insert(point, entity) {return true}
                if subtrees.1.insert(point, entity) {return true}
                if subtrees.2.insert(point, entity) {return true}
                if subtrees.3.insert(point, entity) {return true}
            }
            None => {println!("No subtrees")}
        }

       false
    }

    fn subdivide(&mut self) {
        let half_dimention = self.boundry.half_dimention/2.;
        self.subtrees = Some(Box::new((
            QuadTree::new(AABB::new(
                Vec2::new(
                    self.boundry.center.x + half_dimention, 
                    self.boundry.center.y + half_dimention
                ),
                half_dimention
            )),
            QuadTree::new(AABB::new(
                Vec2::new(
                    self.boundry.center.x - half_dimention, 
                    self.boundry.center.y + half_dimention
                ),
                half_dimention
            )),
            QuadTree::new(AABB::new(
                Vec2::new(
                    self.boundry.center.x + half_dimention, 
                    self.boundry.center.y - half_dimention
                ),
                half_dimention
            )),
            QuadTree::new(AABB::new(
                Vec2::new(
                    self.boundry.center.x - half_dimention, 
                    self.boundry.center.y - half_dimention
                ),
                half_dimention
            )),
        )));

    }

    pub fn query_range(&self, range: &AABB) -> Vec<Entity>{
        let mut points_in_range: Vec<Entity> = vec![];

        if !self.boundry.inersects_aabb(range) {return points_in_range}

        for point in self.points {
            match point {
                Some(point) => {
                    if range.contains_point(point.0) {
                        points_in_range.push(point.1);
                    }
                }
                None => {break}
            }
        }

        match self.subtrees.as_ref() {
            None => {return points_in_range}
            Some(sub_tress) => {
                points_in_range.append(&mut sub_tress.0.query_range(range));
                points_in_range.append(&mut sub_tress.1.query_range(range));
                points_in_range.append(&mut sub_tress.2.query_range(range));
                points_in_range.append(&mut sub_tress.3.query_range(range));
            }
        }

        points_in_range
    }
}


//TODO diferentiate between dynamic and static entities
#[derive(Component)]
pub struct QuadTreeElement;

fn buildtree(
    quad_tree_element_query: Query<(Entity, &Transform), With<QuadTreeElement>>,
    mut quad_tree: ResMut<QuadTree>,
) {
    quad_tree.points = [None, None, None, None];
    quad_tree.i = 0;
    quad_tree.subtrees = None;
    for (entity, transform) in quad_tree_element_query.iter() {
        quad_tree.insert(transform.translation.xy(), entity);
    }
}

// fn render_tree(
//     mut gizmos: Gizmos,
//     quad_tree: Res<QuadTree>,
// ) {
//     render_sub_tree(&mut gizmos, quad_tree.into_inner())
// }

// fn render_sub_tree(
//     gizmos: &mut Gizmos,
//     quad_tree: &QuadTree,
// ) {
//     gizmos.rect_2d(quad_tree.boundry.center, 0., Vec2::new(quad_tree.boundry.half_dimention, quad_tree.boundry.half_dimention)*2., Color::GREEN);
//     if let Some(subtrees) = &quad_tree.subtrees {
//         render_sub_tree(gizmos, &subtrees.0);
//         render_sub_tree(gizmos, &subtrees.1);
//         render_sub_tree(gizmos, &subtrees.2);
//         render_sub_tree(gizmos, &subtrees.3);
//     }
// }