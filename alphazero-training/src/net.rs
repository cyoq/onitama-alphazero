use tch::{
    nn::{self, ConvConfig, ModuleT},
    Tensor,
};

use crate::common::Options;

#[derive(Debug)]
pub struct SmallBlock {
    pub conv: nn::Conv2D,
    pub batch_norm: nn::BatchNorm,
}

impl SmallBlock {
    pub fn new(path: &nn::Path, c_in: i64, c_out: i64) -> Self {
        let conv = nn::conv2d(
            path / "small_block_conv",
            c_in,
            c_out,
            3,
            nn::ConvConfig {
                stride: 1,
                padding: 1,
                ..Default::default()
            },
        );
        let batch_norm = nn::batch_norm2d(path / "small_block_bn", c_out, Default::default());
        Self { conv, batch_norm }
    }
}

impl nn::ModuleT for SmallBlock {
    fn forward_t(&self, xs: &tch::Tensor, train: bool) -> tch::Tensor {
        xs.apply_t(&self.conv, train)
            .apply_t(&self.batch_norm, train)
    }
}

#[derive(Debug)]
pub struct ResNetBlock {
    pub small_block1: SmallBlock,
    pub small_block2: SmallBlock,
}

impl ResNetBlock {
    pub fn new(path: &nn::Path, c_in: i64, c_out: i64) -> Self {
        let small_block1 = SmallBlock::new(&(path / "resnet_small_block1"), c_in, c_out);
        let small_block2 = SmallBlock::new(&(path / "resnet_small_block2"), c_in, c_out);
        Self {
            small_block1,
            small_block2,
        }
    }
}

impl nn::ModuleT for ResNetBlock {
    fn forward_t(&self, xs: &tch::Tensor, train: bool) -> tch::Tensor {
        let mut y = xs
            .apply_t(&self.small_block1, train)
            .relu()
            .apply_t(&self.small_block2, train);
        // skipping layer
        y += xs;
        y.relu()
    }
}

#[derive(Debug)]
pub struct DropoutBlock {
    pub linear: nn::Linear,
    pub batch_norm: nn::BatchNorm,
}

impl DropoutBlock {
    pub fn new(path: &nn::Path, c_in: i64, c_out: i64) -> Self {
        let linear = nn::linear(
            &(path / "droupout_linear"),
            c_in,
            c_out,
            nn::LinearConfig::default(),
        );
        let batch_norm = nn::batch_norm1d(
            &(path / "droupout_bn"),
            c_out,
            nn::BatchNormConfig::default(),
        );
        Self { linear, batch_norm }
    }
}

impl nn::ModuleT for DropoutBlock {
    fn forward_t(&self, xs: &tch::Tensor, train: bool) -> tch::Tensor {
        xs.apply_t(&self.linear, train)
            .apply_t(&self.batch_norm, train)
            .relu()
            .dropout(0.3, train)
    }
}

#[derive(Debug)]
pub struct ResTowerTensor {
    pub policy: Tensor,
    pub value: Tensor,
}

#[derive(Debug, Clone)]
pub struct ConvResNetConfig {
    pub hidden_channels: i64,
    pub input_channels: i64,
    pub resnet_block_amnt: i64,
}

impl Default for ConvResNetConfig {
    fn default() -> Self {
        Self {
            hidden_channels: 32,
            input_channels: 21,
            resnet_block_amnt: 3,
        }
    }
}

#[derive(Debug)]
pub struct ConvResNet {
    pub model: nn::SequentialT,
    pub policy_head: nn::SequentialT,
    pub value_head: nn::SequentialT,
    pub options: Options,
    pub id: String,
}

impl ConvResNet {
    pub fn new(path: &nn::Path, net_config: ConvResNetConfig, options: Options) -> Self {
        let id = format!(
            "conv_input_{}_hidden_{}_resnet_{}",
            net_config.input_channels, net_config.hidden_channels, net_config.resnet_block_amnt
        );
        let policy_head = Self::build_policy_head(path, &net_config, options);
        let value_head = Self::build_value_head(path, &net_config);
        let model = Self::build_model(path, net_config);
        Self {
            model,
            policy_head,
            value_head,
            options,
            id,
        }
    }

    fn build_model(path: &nn::Path, net_config: ConvResNetConfig) -> nn::SequentialT {
        let initial_block = nn::seq_t()
            .add(nn::conv2d(
                &(path / "conv_init_1"),
                net_config.input_channels,
                net_config.hidden_channels,
                3,
                nn::ConvConfig {
                    stride: 1,
                    padding: 1,
                    ..Default::default()
                },
            ))
            .add(nn::batch_norm2d(
                &(path / "bn1"),
                net_config.hidden_channels,
                Default::default(),
            ))
            .add_fn(|xs| xs.relu());

        let mut middle_blocks = nn::seq_t();
        for i in 0..net_config.resnet_block_amnt {
            middle_blocks = middle_blocks.add(ResNetBlock::new(
                &(path / format!("resnet_{}", i)),
                net_config.hidden_channels,
                net_config.hidden_channels,
            ));
        }

        let model = nn::seq_t().add(initial_block).add(middle_blocks);

        model
    }

    fn build_value_head(path: &nn::Path, net_config: &ConvResNetConfig) -> nn::SequentialT {
        nn::seq_t()
            .add(nn::conv2d(
                &(path / "vh_conv"),
                net_config.hidden_channels,
                1,
                1,
                ConvConfig {
                    stride: 1,
                    ..Default::default()
                },
            ))
            .add(nn::batch_norm2d(&(path / "vh_bn"), 1, Default::default()))
            .add_fn(|xs| xs.relu())
            .add_fn(|xs| xs.flatten(1, -1))
            .add(nn::linear(
                &(path / "vh_linear1"),
                25,
                1,
                nn::LinearConfig::default(),
            ))
            .add_fn(|xs| xs.tanh())
    }

    fn build_policy_head(
        path: &nn::Path,
        net_config: &ConvResNetConfig,
        options: Options,
    ) -> nn::SequentialT {
        nn::seq_t()
            .add(nn::conv2d(
                &(path / "policy_conv"),
                net_config.hidden_channels,
                2,
                1,
                ConvConfig {
                    stride: 1,
                    ..Default::default()
                },
            ))
            .add(nn::batch_norm2d(
                &(path / "policy_bn"),
                2,
                Default::default(),
            ))
            .add_fn(|xs| xs.relu())
            .add_fn(|xs| xs.flatten(1, -1))
            .add(nn::linear(
                &(path / "ph_linear2"),
                50,
                50,
                nn::LinearConfig::default(),
            ))
            .add_fn(move |xs| xs.softmax(-1, options.kind))
    }

    pub fn forward(&self, xs: &Tensor, train: bool) -> ResTowerTensor {
        let y: Tensor;
        if !train {
            // Get one more dimension for the Batch
            y = self.model.forward_t(&xs.unsqueeze(0), train);
        } else {
            // For training there should be batch dimension
            y = self.model.forward_t(&xs, train);
        }

        let v = self.value_head.forward_t(&y, train);
        let p = self.policy_head.forward_t(&y, train).reshape(&[-1, 2, 25]);

        ResTowerTensor {
            policy: p,
            value: v,
        }
    }

    pub fn alphaloss(&self, v: &Tensor, p: &Tensor, pi: &Tensor, z: &Tensor) -> (Tensor, Tensor) {
        let diff = z.to_device(self.options.device) - v;
        let value_loss = (&diff * &diff).mean(self.options.kind);

        let policy_loss = -(p.log() * pi)
            .sum_dim_intlist([1].as_slice(), false, self.options.kind)
            .mean(self.options.kind);

        (value_loss, policy_loss)
    }
}
