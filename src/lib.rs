pub mod extractor;

use plotters::prelude::*;

use plotters::style::text_anchor::HPos;
use plotters::style::text_anchor::Pos;
use plotters::style::text_anchor::VPos;
use plotters::style::FontTransform;
use plotters::style::IntoFont;
use plotters::style::TextStyle;
use plotters::style::WHITE;

pub fn plot(name: &str, input: &Vec<extractor::Response>) -> Result<(), Box<dyn std::error::Error>> {

    let max = input.iter().max_by(|x, y| x.exec_time.cmp(&y.exec_time)).unwrap();
    let file_name = format!("{}",input[0].date_as_date.format("%Y-%m-%d.png"));
    let path = String::from("charts/") + &file_name + "_"+ name;
    
    let root = BitMapBackend::new(&path, (1024, 640)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&root)
        .caption(file_name + " " + name, ("sans-serif", 50).into_font())
        .margin(5)
        .x_label_area_size(150)
        .y_label_area_size(150)
        .build_cartesian_2d(input[0].date_as_date..input.last().unwrap().date_as_date, 0f32..(max.exec_time+10) as f32)?;

    
    let pos = Pos::new(HPos::Left, VPos::Bottom);
    let x_label_style = TextStyle::from(("Ubuntu Medium", 10).into_font())
        .pos(pos)
        .transform(FontTransform::Rotate270);

    chart.configure_mesh()
        .x_labels(50)
        .x_label_style(x_label_style)
        .x_label_offset(-10)
        .x_label_formatter(&|pos| {

          let label = format!("{}",pos.format("%H:%M"));
            label
        })
        .draw()?;

    
         chart
        .draw_series(LineSeries::new(
            (0..input.len()).map(|x| (input[x].date_as_date,input[x].exec_time as f32)),
            &RED,
        ))?;
        //.label(name)
        //.legend(move |(x, y)| Rectangle::new([(x, y-10), (x + 10, y+20)], &BLUE));

        chart
        .configure_series_labels()
        //.background_style(&WHITE.mix(0.8))
        //.border_style(&BLACK)
        .draw()?;

    root.present()?;

    Ok(())
}