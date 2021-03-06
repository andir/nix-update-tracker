use horrorshow::prelude::*;

pub trait ValueFunction<R> {
    fn get_value(&self, row: &R) -> Box<dyn Render>;
}

impl<F, R, V> ValueFunction<R> for F
where
    F: 'static + (Fn(&R) -> Box<V>),
    V: Render + 'static,
{
    fn get_value(&self, row: &R) -> Box<dyn Render> {
        let ret: Box<V> = self(row);
        ret
    }
}

//impl<F, R> ValueFunction<R> for F
//    where F: Fn(&R) -> String {
//    fn get_value(&self, row: &R) -> Box<Render> {
//        Box::new(self(row))
//    }
//}

type Func<R> = dyn Fn(&R) -> Box<dyn Render>;

pub struct Column<R> {
    _name: String,
    value_fn: Box<Func<R>>,
}

impl<R> Column<R> {
    pub fn new<S>(name: S, value_fn: &'static Func<R>) -> Column<R>
    where
        S: AsRef<str>,
    {
        Self {
            _name: name.as_ref().to_string(),
            value_fn: Box::new(value_fn),
        }
    }
}

pub trait RenderColumn<R> {
    fn name(&self) -> String;
    fn get_value(&self, row: &R) -> Box<dyn Render>;
}

impl<'call, R> RenderColumn<R> for Column<R> {
    fn name(&self) -> String {
        self._name.clone()
    }

    fn get_value(&self, row: &R) -> Box<dyn Render> {
        let val = (self.value_fn)(row);
        Box::new(val)
    }
}

pub trait AsTable<R, C>
where
    C: RenderColumn<R>,
{
    fn as_table(&self, tmpl: &mut TemplateBuffer, columns: &[C]);
}

impl<T, C> AsTable<T, C> for Vec<T>
where
    C: RenderColumn<T>,
{
    fn as_table(&self, tmpl: &mut TemplateBuffer, columns: &[C]) {
        let headings = columns.iter().map(|c| c.name());
        let rows = self.iter().map(|row| {
            columns
                .iter()
                .map(|column| column.get_value(&row))
                .collect::<Vec<Box<dyn Render>>>()
        });
        tmpl << html! {
            table(class="table") {
                thead {
                    tr {
                        @ for heading in headings {
                            td {
                                : heading;
                            }
                        }
                    }
                }
                tbody {
                    @ for row in rows {
                        tr {
                            @ for column in row.iter() {
                                td {
                                    : column;
                                }
                            }
                        }
                    }
                }
            }
        };
    }
}
//
//fn main() {
//
//    let data = vec![
//        MyRow { name: "foo".to_owned(), value: 123 },
//        MyRow { name: "bar".to_owned(), value: 465 },
//        MyRow { name: "baz".to_owned(), value: 890 },
//    ];
//
//
//    let table = data.as_table(&vec![
//        Column{ _name: "first".to_owned(), value_fn: (|row: &MyRow| row.name.clone()) },
//        Column{ _name: "second".to_owned(), value_fn: (|row: &MyRow| format!("{}", row.value)) }
//    ]);
//
//    println!("{}", table);
//
//}
