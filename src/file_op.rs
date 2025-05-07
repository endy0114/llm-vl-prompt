use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use regex::Regex;
use serde_json::Value;
use tracing::info;
use walkdir::WalkDir;

///
/// 文件操作：
/// 1. 读取配置文件
/// 2. 给图片重新命名
/// 3. 生成推理结果文件
///

pub struct FileOp {
    pub config_path: String,
    pub algorithm_config_path: String,
    pub image_path: String,
    result_file: File,
}

impl FileOp {
    // 创建一个新的 FileOp 实例
    pub fn new(config_path: String, algorithm_config_path: String, image_path: String) -> Self {
        let result_path = Path::new("result");
        if !result_path.exists() {
            fs::create_dir_all(result_path).unwrap();
        }

        let result_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("result/result.txt".to_string())
            .unwrap();
        Self {
            config_path,
            algorithm_config_path,
            image_path,
            result_file,
        }
    }

    // 重命名图片文件
    pub async fn rename_image(&self) -> Result<(), anyhow::Error> {
        // 获取图片列表
        let image_list = self.get_image_list()?;
        // 遍历图片列表，获取图片文件的后缀名，将遍历序号格式化成6位长度的字符串，然后拼接后缀名，生成新的文件名
        for file_info in image_list {
            let file_list = &file_info.1.clone();
            for (i, image_path) in file_list.iter().enumerate() {
                let ext = Path::new(image_path).extension().unwrap().to_str().unwrap();
                let new_name = format!("{:06}.{}", i, ext);
                let new_path = Path::new(&self.image_path).join(&new_name);
                // 使用 fs::rename 函数重命名文件
                fs::rename(image_path, new_path)?;
                println!("Renamed {} to {}", image_path, &new_name); // 打印重命名信息
            }
        }

        Ok(())
    }

    // 读取配置文件
    pub fn read_config(&self) -> Result<String, anyhow::Error> {
        let config_content = fs::read_to_string(&self.config_path)?;
        Ok(config_content)
    }

    // 读取算法配置文件
    pub fn read_algorithm_config(&self) -> Result<String, anyhow::Error> {
        let algorithm_config_content = fs::read_to_string(&self.algorithm_config_path)?;
        Ok(algorithm_config_content)
    }

    pub fn get_image_list(&self) -> Result<Vec<(String, Vec<String>)>, anyhow::Error> {
        let dir = Path::new(self.image_path.as_str());
        let mut image_files: Vec<(String, Vec<String>)> = Vec::new();

        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let mut files = Vec::new();
                    for file in WalkDir::new(path) {
                        let file = file?;
                        if file.file_type().is_file() {
                            files.push(file.path().to_string_lossy().to_string());
                        }
                    }
                    image_files.push((entry.file_name().to_string_lossy().to_string(), files));
                }
            }
        }
        Ok(image_files)
    }

    // 解析并保存推理结果
    pub fn save_inference_result(
        &mut self,
        algorithm: &str,
        image: &str,
        result: &str,
        parse_json: bool,
    ) -> Result<(), anyhow::Error> {
        let pas_result = if parse_json {
            let json = self.parse_json_from_markdown(result)?;
            let mut res = serde_json::from_str::<Value>(&json)?;

            // 如果结果为true，则将图片复制到result目录下
            if res["result"].as_bool().unwrap_or(false) {
                let image_path = Path::new(image);
                let new_image_path = Path::new("result").join(format!(
                    "{}_{}",
                    algorithm,
                    image_path.file_name().unwrap().to_string_lossy()
                ));
                fs::copy(image, new_image_path)?;
            }

            res.as_object_mut().unwrap().insert(
                "algorithm".to_string(),
                Value::String(algorithm.to_string()),
            );
            res.as_object_mut()
                .unwrap()
                .insert("image".to_string(), Value::String(image.to_string()));
            info!("解析的json数据: {:?}", res);
            res.to_string()
        } else {
            format!(
                "algorithm：{:?} image ：{:?} 检测结果:\n{:#?}",
                algorithm, image, result
            )
        };
        self.result_file.write_all(pas_result.as_bytes())?;
        self.result_file.write_all("\n".as_bytes())?;
        Ok(())
    }

    // 解析推理结果里面的json数据
    pub fn parse_json_from_markdown(&self, markdown: &str) -> Result<String, anyhow::Error> {
        info!("Parsing JSON from markdown : \n{}", markdown);
        let re = Regex::new(r#"(?s)```json\s*\n(.*?)\n\s*```"#).unwrap();
        let mut json_list = Vec::new();

        for cap in re.captures_iter(markdown) {
            if let Some(m) = cap.get(1) {
                json_list.push(m.as_str().to_string());
            }
        }
        Ok(json_list.concat())
    }
}

// 测试代码
#[cfg(test)]
mod tests {
    use regex::Regex;

    use super::*;

    #[test]
    fn test_get_image_list() {
        let file_op = FileOp::new(
            "src/config.yaml".to_string(),
            "src/algorithm_config.yaml".to_string(),
            "/home/pengwei/文档/武汉云测试/2025-04-29".to_string(),
        );
        let image_list = file_op.get_image_list().unwrap();
        for (dir, files) in image_list {
            println!("Directory: {}", dir);
            for file in files {
                println!("File: {}", file);
            }
        }
    }

    #[tokio::test]
    async fn test_rename_image() {
        let file_op = FileOp::new(
            "src/config.yaml".to_string(),
            "src/algorithm_config.yaml".to_string(),
            "src/images".to_string(),
        );
        file_op.rename_image().await.unwrap();
    }

    #[test]
    fn test_read_json_from_markdown() {
        let markdown = r#"
               ```json
{
  "result": true,
  "confidence": 0.85
}
```

**理由:**

图片中存在多个明亮的光点，这些光点在夜间城市环境中非常具有烟火的特征。 尽管路灯的光线可能会造成干扰，但这些光点的亮度、分布和闪烁频率都指向烟火的可能性。  置信度为0.85，表示判断存在烟火的概率较高。
                "#;

        let re = Regex::new(r#"(?s)```json\s*\n(.*?)\n\s*```"#).unwrap();
        let mut json_list = Vec::new();

        for cap in re.captures_iter(markdown) {
            if let Some(m) = cap.get(1) {
                json_list.push(m.as_str().to_string());
            }
        }

        println!("提取的 JSON 内容:{}", json_list.concat());
    }
}
