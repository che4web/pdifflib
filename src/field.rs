use ndarray::Array2;

pub struct Field2D {
    name: &'static str,
    pub f: Array2<f64>,
}

impl Field2D {
    pub fn new(name: &'static str, nx: usize, ny: usize) -> Self {
        Self {
            name,
            f: Array2::<f64>::zeros((nx, ny)),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl Field for Field2D {
    fn name(&self) -> &'static str {
        self.name
    }

    fn get_f(&self) -> &Array2<f64> {
        &self.f
    }

    fn get_f_mut(&mut self) -> &mut Array2<f64> {
        &mut self.f
    }
}

#[derive(Copy, Clone)]
pub struct Grid {
    nx: usize,
    ny: usize,
}

impl Grid {
    pub fn new(nx: usize, ny: usize) -> Self {
        Self { nx, ny }
    }

    pub fn inner_nodes(self) -> impl Iterator<Item = (usize, usize)> {
        (1..self.nx - 1).flat_map(move |i| (1..self.ny - 1).map(move |j| (i, j)))
    }

    pub fn x_faces(self) -> impl Iterator<Item = (usize, usize)> {
        (1..self.nx).flat_map(move |i| (0..self.ny).map(move |j| (i, j)))
    }

    pub fn y_faces(self) -> impl Iterator<Item = (usize, usize)> {
        (0..self.nx).flat_map(move |i| (1..self.ny).map(move |j| (i, j)))
    }
}

pub trait Field{
    fn name(&self)-> &'static str;
    fn get_f(&self)-> &Array2<f64>;
    fn get_f_mut(&mut self)-> &mut Array2<f64>;
    unsafe fn mx_b(&self,index:(usize,usize))->f64{
        return (*self.get_f().uget((index.0,index.1))+*self.get_f().uget((index.0-1,index.1)))/2.0
    }

    unsafe fn my_b(&self,index:(usize,usize))->f64{
        return (*self.get_f().uget((index.0,index.1))+*self.get_f().uget((index.0,index.1-1)))/2.0
    }

    unsafe fn dx_f(&self,index:(usize,usize))->f64{
        return *self.get_f().uget((index.0+1,index.1))-*self.get_f().uget((index.0,index.1))
    }

    unsafe fn dx_b(&self,index:(usize,usize))->f64{
        return *self.get_f().uget((index.0,index.1))-*self.get_f().uget((index.0-1,index.1))
    }

    unsafe fn dy_b(&self,index:(usize,usize))->f64{
        return *self.get_f().uget((index.0,index.1))-*self.get_f().uget((index.0,index.1-1))
    }

    unsafe fn dy_f(&self,index:(usize,usize))->f64{
        return *self.get_f().uget((index.0,index.1+1))-*self.get_f().uget((index.0,index.1))
    }

    unsafe fn dx(&self,index:(usize,usize))->f64{
        return (*self.get_f().uget((index.0+1,index.1))-*self.get_f().uget((index.0-1,index.1)))/2.0
    }

    unsafe fn dy(&self,i:(usize,usize))->f64{
        return (*self.get_f().uget((i.0,i.1+1))-*self.get_f().uget((i.0,i.1-1)))/2.0
    }

    unsafe fn lap(&self,i:(usize,usize))->f64{
        return *self.get_f().uget((i.0+1,i.1  ))+
               *self.get_f().uget((i.0-1,i.1  ))+
               *self.get_f().uget((i.0  ,i.1+1))+
               *self.get_f().uget((i.0  ,i.1-1))-
               *self.get_f().uget(i)*4.0;
    }
}


pub unsafe fn dx_b(f:&Array2<f64>,index:(usize,usize))->f64{
    return *f.uget((index.0,index.1))-*f.uget((index.0-1,index.1))
}

pub unsafe fn dy_b(f:&Array2<f64>,index:(usize,usize))->f64{
    return *f.uget((index.0,index.1))-*f.uget((index.0,index.1-1))
}
