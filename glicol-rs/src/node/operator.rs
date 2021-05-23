use dasp_graph::{Buffer, Input, Node};
use dasp_slice::add_in_place;
use apodize;
use super::super::{Pairs, Rule, NodeData, 
    NodeResult, BoxedNodeSend, GlicolNodeData, mono_node, Para};

pub struct MonoSum {}

impl MonoSum {
    pub fn new(paras: &mut Pairs<Rule>) -> NodeResult {
        let inputs: Vec<String> = paras.as_str()
        .split(" ").map(|a|a.to_string()).collect();
        // println!("{:?}", inputs);
        Ok(
            (NodeData::new1(
                BoxedNodeSend::new(
                    Self {}
                )
            ), inputs)
        )
    }
}

impl Node<128> for MonoSum {
    fn process(&mut self, inputs: &[Input<128>], output: &mut [Buffer<128>]) {
        let n = inputs.len();
        // clock input[n-1]
        output[0].silence();

        for i in 0..(n-1) {
            let in_buffer = inputs[i].buffers().clone();
            add_in_place(&mut output[0], &in_buffer[0]);
            // for i in 0..64 {
                // output[0][i] += in_buffer[0][i];
            // }
        }
    }
}

pub struct Mul {
    mul: f32,
    transit_begin: f32,
    transit_end: f32,
    transit_index: usize,
    transit: bool,
    window: Vec<f64>,
    sidechain_ids: Vec<u8>
}
impl Mul {
    pub fn new(mul: &str) -> GlicolNodeData {
        let mut sidechain_ids = vec![];
        let mul = match mul.parse::<f32>() {
            Ok(v) => v,
            Err(_) =>  {sidechain_ids.push(0); 1.0}
        };
        // let mul = match mul {
        //     Para::Number(v) => v,
        //     // Para::Sidechain => {sidechain_ids.push(0); 1.0}
        //     Para::Ref(_) => {sidechain_ids.push(0); 1.0}
        //     _ => unimplemented!()
        // };
        return mono_node!( Self {
            mul,
            transit_begin: 0.0,
            transit_end: 0.0,
            transit_index: 0,
            transit: false,
            window: apodize::hanning_iter(2048).collect::<Vec<f64>>(),
            sidechain_ids
        })
    }
}
impl Node<128> for Mul {
    fn process(&mut self, inputs: &[Input<128>], output: &mut [Buffer<128>]) {

        if self.sidechain_ids.len() == 0 {
            output[0] = inputs[0].buffers()[0].clone();
            output[0].iter_mut().for_each(|s| *s = *s * self.mul as f32);
        } else {
            let buf = &mut inputs[1].buffers();
            let mod_buf = &mut inputs[0].buffers();

            self.transit = self.transit_begin != mod_buf[0][0]
            && mod_buf[0][0] == mod_buf[0][63];

            if self.transit {
                self.transit_end = mod_buf[0][0];
            }

            let distance = self.transit_begin - self.transit_end;

            // println!("{} {} {}", self.transit, self.window[self.transit_index], phase);

            if self.transit_index == 1024 {
                self.transit_index = 0;
                self.transit_begin = self.transit_end.clone();
                self.transit = false;
            }

            for i in 0..128 {
                // output[0][i] = self.window[self.transit_index] as f32;
                // self.transit_index += 1;
                output[0][i] = match self.transit {
                    true => {
                        let phase = self.transit_begin - 
                        self.window[self.transit_index] as f32 * distance;
                        self.transit_index += 1;
                        phase * buf[0][i]
                    },
                    false => {
                        mod_buf[0][i] * buf[0][i]
                    }
                };
            }
        }
    }
}

pub struct Add {
    pub inc: f32,
    sidechain_ids: Vec<u8>
}

impl Add {
    pub fn new(inc: &str) -> GlicolNodeData {
        let mut sidechain_ids = vec![];
        let inc = match inc.parse::<f32>() {
            Ok(v) => v,
            Err(_) =>  {sidechain_ids.push(0); 1.0}
        };
        // let inc = match inc {
        //     Para::Number(v) => v,
        //     // Para::NodeIndex => {sidechain_ids.push(0); 0.0},
        //     Para::Ref(_) => {sidechain_ids.push(0); 0.0},
        //     _ => unimplemented!()
        // };
        return mono_node!( Self {
            inc,
            sidechain_ids
        })
    }
}
impl Node<128> for Add {
    fn process(&mut self, inputs: &[Input<128>], output: &mut [Buffer<128>]) {

        if self.sidechain_ids.len() > 0 {
            // assert!(inputs.len() > 1);
            let buf = &mut inputs[0].buffers();
            let mod_buf = &mut inputs[1].buffers();
            for i in 0..128 {
                output[0][i] = mod_buf[0][i] + buf[0][i];
            }
        } else {
            // assert_eq!(inputs.len(), 1);
            output[0] = inputs[0].buffers()[0].clone();
            output[0].iter_mut().for_each(|s| *s = *s + self.inc as f32);
        }
        // if inputs.len() > 0 {
    }
}