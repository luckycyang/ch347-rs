use ch347_rs::ch347;
use ch347_rs::i2c::I2cbus;
use eframe::{egui, epi};
use mpu6050::*;
use nalgebra::{Matrix3, Vector3};
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug)]
struct Delay;

impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        sleep(Duration::from_nanos(u64::from(ns)));
    }
}

impl<UXX: Into<u64>> embedded_hal_027::blocking::delay::DelayMs<UXX> for Delay
where
    u64: From<UXX>,
{
    fn delay_ms(&mut self, ms: UXX) {
        sleep(Duration::from_millis(u64::from(ms)));
    }
}
struct MpuApp {
    roll: f32,
    pitch: f32,
    yaw: f32,
    rx: Receiver<(f32, f32, f32)>, // 用于接收MPU6050数据的通道
}

impl MpuApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();

        // 启动MPU6050数据采集线程
        thread::spawn(move || {
            let p = ch347::init().unwrap();
            let i2c = I2cbus::new(p.I2C, Default::default());
            let mut delay = Delay;

            let mut mpu = Mpu6050::new(i2c);
            mpu.init(&mut delay).unwrap();

            loop {
                // 获取加速度计角度（roll, pitch）
                let acc_angles = mpu.get_acc_angles().expect("Failed to get acc angles");
                let roll = acc_angles.x;
                let pitch = acc_angles.y;

                // 获取陀螺仪数据（yaw通过积分近似）
                let gyro = mpu.get_gyro().expect("Failed to get gyro");
                let yaw_rate = gyro.z;
                static mut YAW: f32 = 0.0;
                unsafe {
                    YAW += yaw_rate * 0.01; // 假设循环时间为10ms
                }

                // 发送数据到主线程
                tx.send((roll, pitch, unsafe { YAW }))
                    .expect("Failed to send data");

                thread::sleep(Duration::from_millis(10));
            }
        });

        Self {
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            rx,
        }
    }

    // 计算旋转矩阵
    fn rotation_matrix(&self) -> Matrix3<f32> {
        let (s_r, c_r) = self.roll.to_radians().sin_cos();
        let (s_p, c_p) = self.pitch.to_radians().sin_cos();
        let (s_y, c_y) = self.yaw.to_radians().sin_cos();

        // 绕Z（yaw） -> 绕Y（pitch） -> 绕X（roll）的旋转矩阵
        Matrix3::new(
            c_y * c_p,
            c_y * s_p * s_r - s_y * c_r,
            c_y * s_p * c_r + s_y * s_r,
            s_y * c_p,
            s_y * s_p * s_r + c_y * c_r,
            s_y * s_p * c_r - c_y * s_r,
            -s_p,
            c_p * s_r,
            c_p * c_r,
        )
    }
}

impl eframe::App for MpuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 从通道接收MPU6050数据
        while let Ok((roll, pitch, yaw)) = self.rx.try_recv() {
            self.roll = roll;
            self.pitch = pitch;
            self.yaw = yaw;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MPU6050 3D Cube Visualization");

            // 显示欧拉角
            ui.label(format!(
                "Roll: {:.2}°, Pitch: {:.2}°, Yaw: {:.2}°",
                self.roll, self.pitch, self.yaw
            ));

            // 3D视图
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, _response) =
                    ui.allocate_exact_size(egui::Vec2::new(400.0, 400.0), egui::Sense::hover());

                let painter = ui.painter();

                // 定义立方体顶点（单位立方体，中心在原点）
                let cube_vertices = [
                    Vector3::new(-0.5, -0.5, -0.5),
                    Vector3::new(0.5, -0.5, -0.5),
                    Vector3::new(0.5, 0.5, -0.5),
                    Vector3::new(-0.5, 0.5, -0.5),
                    Vector3::new(-0.5, -0.5, 0.5),
                    Vector3::new(0.5, -0.5, 0.5),
                    Vector3::new(0.5, 0.5, 0.5),
                    Vector3::new(-0.5, 0.5, 0.5),
                ];

                // 定义立方体边（连接的顶点对）
                let edges = [
                    (0, 1),
                    (1, 2),
                    (2, 3),
                    (3, 0), // 底面
                    (4, 5),
                    (5, 6),
                    (6, 7),
                    (7, 4), // 顶面
                    (0, 4),
                    (1, 5),
                    (2, 6),
                    (3, 7), // 侧边
                ];

                // 旋转立方体顶点
                let rot_matrix = self.rotation_matrix();
                let rotated_vertices: Vec<Vector3<f32>> =
                    cube_vertices.iter().map(|v| rot_matrix * v).collect();

                // 透视投影参数
                let fov = 90.0f32.to_radians();
                let aspect = rect.width() / rect.height();
                let near = 0.1;
                let far = 100.0;
                let scale = (fov / 2.0).tan() * near;
                let proj_matrix = Matrix3::new(
                    near / (scale * aspect),
                    0.0,
                    0.0,
                    0.0,
                    near / scale,
                    0.0,
                    0.0,
                    0.0,
                    -(far + near) / (far - near),
                );

                // 投影到2D平面
                let projected: Vec<egui::Pos2> = rotated_vertices
                    .iter()
                    .map(|v| {
                        let v = Vector3::new(v.x, v.y, v.z + 2.0); // 移动到相机前方
                        let proj = proj_matrix * v;
                        let x = proj.x / proj.z;
                        let y = proj.y / proj.z;
                        egui::Pos2::new(
                            rect.center().x + x * rect.width() / 2.0,
                            rect.center().y - y * rect.height() / 2.0, // Y轴向上
                        )
                    })
                    .collect();

                // 绘制立方体边
                for &(i, j) in edges.iter() {
                    painter.line_segment(
                        [projected[i], projected[j]],
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
                }
            });
        });

        // 请求重绘以保持动画流畅
        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "MPU6050 Cube",
        options,
        Box::new(|cc| Box::new(MpuApp::new(cc))),
    )
}
