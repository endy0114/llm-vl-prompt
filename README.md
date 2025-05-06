## 工具作用

**目标：** 让大家能够使用命令行快速的优化和测试多模态大模型进行图片识别的prompt


## 工具使用

```shell

Usage: lvt.exe [OPTIONS]

Options:
  -c, --config <CONFIG>                      [default: config/config.json]     // 著配置文件路径，不需要指定，按照需求修改即可
  -a, --algorithm-config <ALGORITHM_CONFIG>  [default: config/algorithm.json]  // 算法定义，不需要指定，按照需求修改即可
  -i, --image-path <IMAGE_PATH>              [default: images]                 // 图片文件的路径，需要指定
  -r, --rename                                                                 // 对图片进行重命名，默认否
  -p, --parse-json                                                             // 是否解析推理结果里面的json数据，默认否
  -h, --help                                 Print help
  -V, --version                              Print version

```

**使用示例：** `lvt lvt.exe -i D:\workspace\2025-04-29 -p`
命令执行完成后，会在命令窗口所在的路径创建**result**文件夹，将检测为**true**的图片复制到文件夹下面，并生成**result.txt**结果文件

## 检测提示词编写技巧

> 使用大模型进行安防检测时，提示词非常的重要，往往一张图片里面包含的信息非常的多。大模型在做检测识别的时候其实是没有什么方向的。我们为其写提示词就是让大模型的检测围绕我们的目标进行。
> 多数情况下，我们能够清楚的描述我们的目标，但是缺乏对目标关联因素的描述，所有往往都不能取得较好的效果。下面是一些技巧，可以参考一下。
> 1. 提示词要尽量简洁，不要包含无关信息。
> 2. 提示词要尽量具体，不要使用模糊的词汇。
> 3. 目标要明确不要让大模型去猜测。
> 4. 影响目标识别的关联因素要尽量描述清楚。
