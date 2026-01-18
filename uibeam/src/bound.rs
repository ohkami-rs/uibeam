
use crate::Beam;

pub trait BeamKind {}
pub struct Server;
#[cfg(feature = "client")]
pub struct Client;
impl BeamKind for Server {}
#[cfg(feature = "client")]
impl BeamKind for Client {}

#[cfg(feature = "client")]
pub trait IslandBoundary:
    Beam<Client> + serde::Serialize + for<'de> serde::Deserialize<'de>
{
}

pub struct ServerOrIslandBoundary<K: BeamKind>(std::marker::PhantomData<K>);
#[cfg(feature = "client")]
pub struct Anywhere<K: BeamKind>(std::marker::PhantomData<K>);
impl<K: BeamKind> BeamKind for ServerOrIslandBoundary<K> {}
#[cfg(feature = "client")]
impl<K: BeamKind> BeamKind for Anywhere<K> {}

// `ServerOrIslandBoundary` means either server Beam or island-boundary client Beam that is Serialize-able.
impl<T> Beam<ServerOrIslandBoundary<Server>> for T
where
    T: Beam<Server>,
{
    #[inline(always)]
    fn render(self) -> super::UI {
        Beam::<Server>::render(self)
    }
}
#[cfg(feature = "client")]
impl<T> Beam<ServerOrIslandBoundary<Client>> for T
where
    T: IslandBoundary,
{
    #[inline(always)]
    fn render(self) -> super::UI {
        Beam::<Client>::render(self)
    }
}

#[cfg(feature = "client")]
impl<T> Beam<Anywhere<Server>> for T
where
    T: Beam<Server>,
{
    #[inline(always)]
    fn render(self) -> super::UI {
        Beam::<Server>::render(self)
    }
}
#[cfg(feature = "client")]
impl<T> Beam<Anywhere<Client>> for T
where
    T: Beam<Client>,
{
    #[inline(always)]
    fn render(self) -> super::UI {
        Beam::<Client>::render(self)
    }
}

#[doc(hidden)]
pub fn render_on_server<K: BeamKind>(beam: impl Beam<ServerOrIslandBoundary<K>>) -> super::UI {
    Beam::<ServerOrIslandBoundary<K>>::render(beam)
}
#[cfg(feature = "client")]
#[doc(hidden)]
pub fn render_in_island<K: BeamKind>(beam: impl Beam<Anywhere<K>>) -> super::UI {
    Beam::<Anywhere<K>>::render(beam)
}
