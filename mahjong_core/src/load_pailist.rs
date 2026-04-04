use std::{fs::File, path::Path};

use anyhow::Context;
use arrow_array::{
    array::Int32Array, Array, FixedSizeListArray, ListArray, RecordBatch, StringArray, StructArray,
    UInt32Array,
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::mahjong_generated::open_mahjong::{Mentsu, MentsuFlag, MentsuT, MentsuType, PaiT};

#[derive(Default, Debug)]
pub struct ParquetAgari {
    pub tehai: Vec<PaiT>,
    pub fulo: Vec<Mentsu>,
    pub yaku: Vec<(String, i32)>,
    pub fu: i32,
    pub han: i32,
    pub score: i32,
    pub machipai: PaiT,
    pub dora: Vec<PaiT>,
    pub uradora: Vec<PaiT>,
    pub nukidora: u32,
}

fn str_to_pais<T: AsRef<str>>(input: T) -> Vec<PaiT> {
    let s = input.as_ref();
    let mut ret: Vec<PaiT> = Vec::new();
    let mut suit: u8 = 0;
    for c in s.chars() {
        match c {
            'm' => suit = 0,
            'p' => suit = 1,
            's' => suit = 2,
            'z' => suit = 3,
            _ => {
                let pai = PaiT {
                    pai_num: suit * 9 + c.to_digit(10).unwrap() as u8 - 1,
                    id: 0,
                    is_nakare: false,
                    is_riichi: false,
                    is_tsumogiri: false,
                };
                ret.push(pai);
            }
        }
    }

    ret
}

fn str_to_fulo(s: &str) -> Mentsu {
    let mut ret: MentsuT = Default::default();
    let mut suit: u8 = 0;
    let mut index: usize = 0;
    let mut naki = false;
    for c in s.chars() {
        match c {
            'm' => suit = 0,
            'p' => suit = 1,
            's' => suit = 2,
            'z' => suit = 3,
            '-' => {
                ret.pai_list[index - 1].flag = MentsuFlag::FLAG_KAMICHA;
                naki = true;
            }
            '=' => {
                ret.pai_list[index - 1].flag = MentsuFlag::FLAG_TOIMEN;
                naki = true;
            }
            '+' => {
                ret.pai_list[index - 1].flag = MentsuFlag::FLAG_SIMOCHA;
                naki = true;
            }
            _ => {
                let pai_num = suit * 9 + c.to_digit(10).unwrap() as u8 - 1;
                ret.pai_list[index].pai_num = pai_num;
                index += 1;
            }
        }
    }

    if index == 4 {
        ret.mentsu_type = if naki {
            MentsuType::TYPE_MINKAN
        } else {
            MentsuType::TYPE_ANKAN
        };
    } else if ret.pai_list[0].pai_num == ret.pai_list[1].pai_num {
        ret.mentsu_type = MentsuType::TYPE_KOUTSU;
    } else {
        ret.mentsu_type = MentsuType::TYPE_SHUNTSU;
    }

    ret.pack()
}

impl ParquetAgari {
    pub fn parse_tehai_string(&mut self, s: &str) {
        let mut part = s.split(",");

        let tehai = part.next().unwrap();

        self.tehai = str_to_pais(tehai);

        for hand in part {
            self.fulo.push(str_to_fulo(hand));
        }
    }

    pub fn get_row_with_types(
        &mut self,
        record_batch: &RecordBatch,
        row_index: usize,
    ) -> anyhow::Result<()> {
        for (i, column) in record_batch.columns().iter().enumerate() {
            let binding = record_batch.schema();
            let field = binding.field(i);
            let name = field.name();

            if name == "tehai" {
                let string_array = column
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .context("tehai column is not StringArray")?;
                let cell = string_array.value(row_index);
                self.parse_tehai_string(cell);
            }

            if name == "fu" {
                let int_array = column
                    .as_any()
                    .downcast_ref::<Int32Array>()
                    .context("fu column is not Int32Array")?;
                self.fu = int_array.value(row_index);
            }

            if name == "yaku" {
                let string_array = column
                    .as_any()
                    .downcast_ref::<ListArray>()
                    .context("yaku column is not ListArray")?;
                let cell = string_array.value(row_index);

                let yaku_array = cell
                    .as_any()
                    .downcast_ref::<StructArray>()
                    .context("yaku cell is not StructArray")?;

                let yaku_names = yaku_array
                    .column(0)
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .context("yaku_names column is not StringArray")?;
                let yaku_hans = yaku_array
                    .column(1)
                    .as_any()
                    .downcast_ref::<Int32Array>()
                    .context("yaku_hans column is not Int32Array")?;

                for i in 0..yaku_array.len() {
                    let yaku_name = yaku_names.value(i);
                    let yaku_han = yaku_hans.value(i);

                    self.yaku.push((String::from(yaku_name), yaku_han));
                }
            }

            if name == "han" {
                let int_array = column
                    .as_any()
                    .downcast_ref::<Int32Array>()
                    .context("han column is not Int32Array")?;
                self.han = int_array.value(row_index);
            }

            if name == "score" {
                let int_array = column
                    .as_any()
                    .downcast_ref::<Int32Array>()
                    .context("score column is not Int32Array")?;
                self.score = int_array.value(row_index);
            }

            if name == "dora_orig" {
                let string_array = column
                    .as_any()
                    .downcast_ref::<ListArray>()
                    .context("dora_orig column is not ListArray")?;
                let cell = string_array.value(row_index);

                let dora_array = cell
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .context("dora_orig cell is not UInt32Array")?;

                self.dora = (0..dora_array.len())
                    .map(|idx| {
                        let dora = dora_array.value(idx);
                        PaiT {
                            pai_num: (dora >> 2) as u8,
                            id: (dora & 3) as u8,
                            is_nakare: false,
                            is_riichi: false,
                            is_tsumogiri: false,
                        }
                    })
                    .collect();
            }

            if name == "machipai" {
                let int_array = column
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .context("machipai column is not UInt32Array")?;
                let cell = int_array.value(row_index);

                self.machipai = PaiT {
                    pai_num: (cell >> 2) as u8,
                    id: (cell & 3) as u8,
                    is_nakare: false,
                    is_riichi: false,
                    is_tsumogiri: false,
                };
            }

            if name == "uradora_orig" {
                let string_array = column
                    .as_any()
                    .downcast_ref::<ListArray>()
                    .context("uradora_orig column is not ListArray")?;
                let cell = string_array.value(row_index);

                let dora_array = cell
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .context("uradora_orig cell is not UInt32Array")?;

                self.uradora = (0..dora_array.len())
                    .map(|idx| {
                        let dora = dora_array.value(idx);
                        PaiT {
                            pai_num: (dora >> 2) as u8,
                            id: (dora & 3) as u8,
                            is_nakare: false,
                            is_riichi: false,
                            is_tsumogiri: false,
                        }
                    })
                    .collect();
            }

            if name == "nukidora" {
                let int_array = column
                    .as_any()
                    .downcast_ref::<UInt32Array>()
                    .context("nukidora column is not UInt32Array")?;
                let cell = int_array.value(row_index);

                self.nukidora = cell;
            }
        }
        Ok(())
    }
}

pub fn load_pailist<P: AsRef<Path>>(path: P, row_index: usize) -> anyhow::Result<Vec<u32>> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let reader = builder.build()?;
    let mut current_offset = 0;

    for read_result in reader {
        let record_batch = read_result?;
        let batch_len = record_batch.num_rows();

        if row_index >= current_offset && row_index < current_offset + batch_len {
            let local_row_index = row_index - current_offset;

            if let Some(column) = record_batch.column_by_name("pai_ids") {
                if let Some(row_list) = column.as_any().downcast_ref::<FixedSizeListArray>() {
                    let cell = row_list.value(local_row_index);
                    if let Some(row) = cell.as_any().downcast_ref::<UInt32Array>() {
                        return Ok(row.values().to_vec());
                    } else {
                        anyhow::bail!("cannot read cell data");
                    }
                } else {
                    anyhow::bail!("cannot read columns by list");
                }
            } else {
                anyhow::bail!("cannot load pai_ids column");
            }
        }
        current_offset += batch_len;
    }

    anyhow::bail!("row_index {} is out of bounds", row_index);
}

pub fn load_agari_tehai<P: AsRef<Path>>(path: P, row_index: usize) -> anyhow::Result<ParquetAgari> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let reader = builder.build()?;
    let mut ret = ParquetAgari::default();
    let mut current_offset = 0;

    for read_result in reader {
        let record_batch = read_result?;
        let batch_len = record_batch.num_rows();

        if row_index >= current_offset && row_index < current_offset + batch_len {
            let local_row_index = row_index - current_offset;
            ret.get_row_with_types(&record_batch, local_row_index)?;
            return Ok(ret);
        }
        current_offset += batch_len;
    }

    anyhow::bail!("row_index {} is out of bounds", row_index);
}
