use clap::Parser;
use tracing::info;

mod cli;
mod file_op;
mod llm;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    let exec_dir = std::env::current_exe()?;
    let conf_prefix = exec_dir.parent().unwrap().to_str().unwrap();
    info!("当前配置目录: {:?}", conf_prefix);

    let cli = cli::Cli::parse();

    // 构建文件操作对象
    // 绝对定位
    // let mut file_op = file_op::FileOp::new(
    //     format!("{}/{}", conf_prefix, cli.config.unwrap()),
    //     format!("{}/{}", conf_prefix, cli.algorithm_config.unwrap()),
    //     cli.image_path.unwrap(),
    // );

    let config_path = cli.config_path.unwrap();
    let mut file_op = file_op::FileOp::new(
        format!("{}/{}", config_path, "config.json"),
        format!("{}/{}", config_path, "algorithm.json"),
        cli.image_path.unwrap(),
    );

    info!("文件操作对象构建成功, 配置文件路径: {:?}", config_path);

    if cli.rename {
        // 重命名图片
        file_op.rename_image().await?;
        info!("图片重命名成功");
    } else {
        // 构建推理对象
        let llm = llm::LLM::new(&file_op);
        info!("推理对象构建成功");
        // 获取图片列表
        let images = file_op.get_image_list()?;
        let algs: Vec<String> = images.iter().map(|i| i.0.clone()).collect();
        info!("图片列表获取成功:{:?}", algs);
        info!("加载了{}个算法", &llm.algorithms.len());

        // 根据定义的算法进行图片推理
        for (algorithm, image_list) in images {
            // 判断算法是否定义
            if llm.check_algorithm(&algorithm) {
                let llm_client = llm.build_agent(&algorithm)?;
                for image in image_list {
                    let resp = llm.chat(&image, &llm_client).await?;
                    file_op.save_inference_result(&algorithm, &image, &resp, cli.parse_json)?;
                }
            }
        }
    }

    Ok(())
}
