#[derive(Debug, Clone, Default)]
///
/// Exchange options
///
pub enum Exchange {
  #[default]
  NSE,
  NFO,
  CDS,
  BSE,
  BFO,
  BCD,
  MCX,
  MCXSX,
  INDICES,
}

impl Exchange {
  pub(crate) fn divisor(&self) -> f64 {
    match self {
      Self::CDS => 100_000_0.0,
      Self::BCD => 100_0.0,
      _ => 100.0,
    }
  }

  pub(crate) fn is_tradable(&self) -> bool {
    match self {
      Self::INDICES => false,
      _ => true,
    }
  }
}

impl From<usize> for Exchange {
  fn from(value: usize) -> Self {
    match value {
      9 => Self::INDICES,
      8 => Self::MCXSX,
      7 => Self::MCX,
      6 => Self::BCD,
      5 => Self::BFO,
      4 => Self::BSE,
      3 => Self::CDS,
      2 => Self::NFO,
      1 => Self::NSE,
      _ => Self::NSE,
    }
  }
}

impl From<String> for Exchange {
    fn from(value: String) -> Self {
        match value.as_str() {
            "NSE" => Self::NSE,
            "NFO" => Self::NFO,
            "CDS" => Self::CDS,
            "BSE" => Self::BSE,
            "BFO" => Self::BFO,
            "BCD" => Self::BCD,
            "MCX" => Self::MCX,
            "MCXSX" => Self::MCXSX,
            "INDICES" => Self::INDICES,
            _ => Self::NSE,

        }
    }
}

impl From<Exchange> for String {
  fn from(value: Exchange) -> Self {
    match value {
      Exchange::NSE => "NSE".to_string(),
      Exchange::NFO => "NFO".to_string(),
      Exchange::CDS => "CDS".to_string(),
      Exchange::BSE => "BSE".to_string(),
      Exchange::BFO => "BFO".to_string(),
      Exchange::BCD => "BCD".to_string(),
      Exchange::MCX => "MCX".to_string(),
      Exchange::MCXSX => "MCXSX".to_string(),
      Exchange::INDICES => "INDICES".to_string(),
    }
  }
}
