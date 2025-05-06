use clap::Parser;

#[derive(Parser,Debug)]
#[command(version, author, about, long_about = None)]
pub struct Cli{
    // 模型参数配置文件路径，默认为config/config.json
    #[arg(short, long, default_value = "config/config.json")]
    pub config: Option<String>,

    // 算法参数配置文件路径，默认为config/algorithm.json
    #[arg(short, long, default_value = "config/algorithm.json")]
    pub algorithm_config: Option<String>,

    // 图片文件路径
    #[arg(short, long,default_value = "images")]
    pub image_path: Option<String>,

    // 是否给文件重命名，默认为false
    #[arg(short, long, default_value_t = false)]
    pub rename: bool,

    // 是否给文件重命名，默认为false
    #[arg(short, long, default_value_t = false)]
    pub parse_json: bool,

}