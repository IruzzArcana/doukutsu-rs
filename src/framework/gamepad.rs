use std::collections::{HashMap, HashSet};

use sdl2::controller::GameController;
use serde::{Deserialize, Serialize};

use crate::{framework::context::Context, settings::PlayerControllerInputType};

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[repr(u32)]
pub enum Axis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    TriggerLeft,
    TriggerRight,
}

#[derive(Clone, Debug)]
pub enum AxisDirection {
    None,
    Either,
    Up,
    Left,
    Right,
    Down,
}

impl AxisDirection {
    pub fn compare(&self, value: f64, axis_sensitivity: f64) -> bool {
        match self {
            AxisDirection::None => false,
            AxisDirection::Either => value.abs() > 0.0,
            AxisDirection::Down | AxisDirection::Right => value > axis_sensitivity,
            AxisDirection::Up | AxisDirection::Left => value < -axis_sensitivity,
        }
    }
}

#[derive(Debug, Hash, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[repr(u32)]
pub enum Button {
    South,
    East,
    West,
    North,

    Back,
    Guide,
    Start,
    LeftStick,
    RightStick,
    LeftShoulder,
    RightShoulder,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

pub struct GamepadData {
    controller: GameController,

    left_x: f64,
    left_y: f64,
    right_x: f64,
    right_y: f64,
    trigger_left: f64,
    trigger_right: f64,

    axis_sensitivity: f64,

    pressed_buttons_set: HashSet<Button>,
    axis_values: HashMap<Axis, f64>,
}

impl GamepadData {
    pub(crate) fn new(game_controller: GameController, axis_sensitivity: f64) -> Self {
        GamepadData {
            controller: game_controller,

            left_x: 0.0,
            left_y: 0.0,
            right_x: 0.0,
            right_y: 0.0,
            trigger_left: 0.0,
            trigger_right: 0.0,

            axis_sensitivity,

            pressed_buttons_set: HashSet::with_capacity(16),
            axis_values: HashMap::with_capacity(8),
        }
    }
}

pub struct GamepadContext {
    gamepads: Vec<GamepadData>,
}

impl GamepadContext {
    pub(crate) fn new() -> Self {
        Self { gamepads: Vec::new() }
    }

    fn get_gamepad(&self, gamepad_id: u32) -> Option<&GamepadData> {
        self.gamepads.iter().find(|gamepad| gamepad.controller.instance_id() == gamepad_id)
    }

    fn get_gamepad_by_index(&self, gamepad_index: usize) -> Option<&GamepadData> {
        self.gamepads.get(gamepad_index)
    }

    fn get_gamepad_mut(&mut self, gamepad_id: u32) -> Option<&mut GamepadData> {
        self.gamepads.iter_mut().find(|gamepad| gamepad.controller.instance_id() == gamepad_id)
    }

    pub(crate) fn add_gamepad(&mut self, game_controller: GameController, axis_sensitivity: f64) {
        self.gamepads.push(GamepadData::new(game_controller, axis_sensitivity));
    }

    pub(crate) fn remove_gamepad(&mut self, gamepad_id: u32) {
        self.gamepads.retain(|data| data.controller.instance_id() != gamepad_id);
    }

    pub(crate) fn set_button(&mut self, gamepad_id: u32, button: Button, pressed: bool) {
        if let Some(gamepad) = self.get_gamepad_mut(gamepad_id) {
            if pressed {
                gamepad.pressed_buttons_set.insert(button);
            } else {
                gamepad.pressed_buttons_set.remove(&button);
            }
        }
    }

    pub(crate) fn set_axis_value(&mut self, gamepad_id: u32, axis: Axis, value: f64) {
        if let Some(gamepad) = self.get_gamepad_mut(gamepad_id) {
            gamepad.axis_values.insert(axis, value);
        }
    }

    pub(crate) fn is_active(
        &self,
        gamepad_index: u32,
        input_type: &PlayerControllerInputType,
        axis_direction: AxisDirection,
    ) -> bool {
        match input_type {
            PlayerControllerInputType::ButtonInput(button) => self.is_button_active(gamepad_index, *button),
            PlayerControllerInputType::AxisInput(axis) => self.is_axis_active(gamepad_index, *axis, axis_direction),
            PlayerControllerInputType::Either(button, axis) => {
                self.is_button_active(gamepad_index, *button)
                    || self.is_axis_active(gamepad_index, *axis, axis_direction)
            }
        }
    }

    pub(crate) fn is_button_active(&self, gamepad_index: u32, button: Button) -> bool {
        if let Some(gamepad) = self.get_gamepad_by_index(gamepad_index as usize) {
            return gamepad.pressed_buttons_set.contains(&button);
        }

        false
    }

    pub(crate) fn is_axis_active(&self, gamepad_index: u32, axis: Axis, direction: AxisDirection) -> bool {
        if let Some(gamepad) = self.get_gamepad_by_index(gamepad_index as usize) {
            return match axis {
                Axis::LeftX => direction.compare(gamepad.left_x, gamepad.axis_sensitivity),
                Axis::LeftY => direction.compare(gamepad.left_y, gamepad.axis_sensitivity),
                Axis::RightX => direction.compare(gamepad.right_x, gamepad.axis_sensitivity),
                Axis::RightY => direction.compare(gamepad.right_y, gamepad.axis_sensitivity),
                Axis::TriggerLeft => direction.compare(gamepad.trigger_left, 0.0),
                Axis::TriggerRight => direction.compare(gamepad.trigger_right, 0.0),
            };
        }

        false
    }

    pub(crate) fn update_axes(&mut self, gamepad_id: u32) {
        if let Some(gamepad) = self.get_gamepad_mut(gamepad_id) {
            let mut axes = [
                (&mut gamepad.left_x, Axis::LeftX),
                (&mut gamepad.left_y, Axis::LeftY),
                (&mut gamepad.right_x, Axis::RightX),
                (&mut gamepad.right_y, Axis::RightY),
                (&mut gamepad.trigger_left, Axis::TriggerLeft),
                (&mut gamepad.trigger_right, Axis::TriggerRight),
            ];

            for (axis_val, id) in axes.iter_mut() {
                if let Some(axis) = gamepad.axis_values.get(id) {
                    **axis_val = if axis.abs() < 0.12 { 0.0 } else { *axis };
                }
            }
        }
    }
}

impl Default for GamepadContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn add_gamepad(context: &mut Context, game_controller: GameController, axis_sensitivity: f64) {
    context.gamepad_context.add_gamepad(game_controller, axis_sensitivity);
}

pub fn remove_gamepad(context: &mut Context, gamepad_id: u32) {
    context.gamepad_context.remove_gamepad(gamepad_id);
}

pub fn is_active(
    ctx: &Context,
    gamepad_index: u32,
    input_type: &PlayerControllerInputType,
    axis_direction: AxisDirection,
) -> bool {
    ctx.gamepad_context.is_active(gamepad_index, input_type, axis_direction)
}

pub fn is_button_active(ctx: &Context, gamepad_index: u32, button: Button) -> bool {
    ctx.gamepad_context.is_button_active(gamepad_index, button)
}

pub fn is_axis_active(ctx: &Context, gamepad_index: u32, axis: Axis, direction: AxisDirection) -> bool {
    ctx.gamepad_context.is_axis_active(gamepad_index, axis, direction)
}
