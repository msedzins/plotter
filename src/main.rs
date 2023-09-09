use clap::Parser;
use log_plotter::extractor::RestProxyLog;
use log_plotter::extractor::Response;

/// Simple program to plot Rest Proxy log files
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// Return data only (no plotting)
    #[arg(long)]
    data_only: bool,

    /// List of filters used to filter out Rest Proxy log
    #[arg(short, long)]
    filter: Vec<String>,

    /// Position of date inside logs (34m2023-08-28)
    #[arg(short, long, default_value_t = 1)]
    date_pos: usize,

    /// Position of time inside logs (07:01:12.872)
    #[arg(short, long, default_value_t = 2)]
    time_pos: usize,
     
    /// Position of execution time inside logs (2.924797516s)
    #[arg(short, long, default_value_t = 15)]
    exec_pos: usize,

    /// Time window used for calculating the averages
    #[arg(short, long, default_value_t = 5)]
    step: usize,
    
    /// List of REST PROXY log files
    #[arg(last = true)]
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let rp = RestProxyLog::new(args.files, args.filter.clone(), args.date_pos, args.time_pos, args.exec_pos);
    let response = rp.get();
    if args.data_only {
         for v in response.iter() {
            println!("{} {} {}", v.date_as_string, v.date_as_timestamp, v.exec_time)
         }
         return;
    }

    //Calculate averages (should be moved to separate function)
    let mut av: Vec<Response> = Vec::new();
    let mut start = 0;
    let step = args.step;
    loop { 
        let mut end = start + step -1;
        if end >= response.len() {
            end = response.len() -1;
        }

        if start >= end {
            break;
        }

       let average = (start..end).into_iter().fold(0, |acc, x| acc + response[x].exec_time) / (start..end).into_iter().len() as i32;

       let s2 =  if start + step/2 >= response.len() {
            response.len() - 1
        } else  { 
        start + step/2
        };

       av.push(Response {
         date_as_date:  response[s2].date_as_date,
         date_as_timestamp: response[s2].date_as_timestamp,
         exec_time: average,

         date_as_string: response[s2].date_as_string.clone(),
       });

        start += step;
        if start >= response.len() {
            break;
        }
    }

    log_plotter::plot(args.filter.last().unwrap(),&av).expect("Error generating diagram");
}
