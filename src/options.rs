use clap::Clap;

#[derive(Clap)]
pub struct Options {
    #[clap(short, default_value = "fractal.png")]
    pub output: String,

    #[clap(short, default_value = "1024")]
    pub width: u32,
    #[clap(short, default_value = "1024")]
    pub height: u32,

    #[clap(long, default_value = "-0.5557506")]
    pub centre_real: f64,
    #[clap(long, default_value = "-0.55560")]
    pub centre_imaginary: f64,

    #[clap(short, default_value = "20.0")]
    pub zoom: f64,

    #[clap(short, default_value = "50")]
    pub samples_per_pixel: u32,

    #[clap(short, default_value = "4")]
    pub threads: u32,
}
