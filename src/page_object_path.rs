//! Defines the [PdfPagePathObject] struct, exposing functionality related to a single
//! page object defining a path.

use crate::bindgen::{
    FPDF_BOOL, FPDF_FILLMODE_ALTERNATE, FPDF_FILLMODE_NONE, FPDF_FILLMODE_WINDING, FPDF_PAGE,
    FPDF_PAGEOBJECT,
};
use crate::bindings::PdfiumLibraryBindings;
use crate::color::PdfColor;
use crate::error::{PdfiumError, PdfiumInternalError};
use crate::page::{PdfPoints, PdfRect};
use crate::page_object::PdfPageObjectCommon;
use crate::page_object_private::internal::PdfPageObjectPrivate;
use crate::prelude::PdfDocument;
use std::os::raw::{c_int, c_uint};

/// Sets the method used to determine the path region to fill.
///
/// The default fill mode used by `pdfium-render` when creating new [PdfPagePathObject]
/// instances is [PdfPathFillMode::Winding]. The fill mode can be changed on an
/// object-by-object basis by calling the [PdfPagePathObject::set_fill_and_stroke_mode()] function.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PdfPathFillMode {
    /// The path will not be filled.
    None = FPDF_FILLMODE_NONE as isize,

    /// The even-odd rule will be used to determine the path region to fill.
    ///
    /// The even-odd rule determines whether a point is inside a path by drawing a ray from that
    /// point in any direction and simply counting the number of path segments that cross the
    /// ray, regardless of direction. If this number is odd, the point is inside; if even, the
    /// point is outside. This yields the same results as the nonzero winding number rule
    /// for paths with simple shapes, but produces different results for more complex shapes.
    ///
    /// More information, including visual examples, can be found in Section 4.4.2 of
    /// the PDF Reference Manual, version 1.7, on page 233.
    EvenOdd = FPDF_FILLMODE_ALTERNATE as isize,

    /// The non-zero winding number rule will be used to determine the path region to fill.
    ///
    /// The nonzero winding number rule determines whether a given point is inside a
    /// path by conceptually drawing a ray from that point to infinity in any direction
    /// and then examining the places where a segment of the path crosses the ray. Start-
    /// ing with a count of 0, the rule adds 1 each time a path segment crosses the ray
    /// from left to right and subtracts 1 each time a segment crosses from right to left.
    /// After counting all the crossings, if the result is 0, the point is outside the path;
    /// otherwise, it is inside.
    ///
    /// This is the default fill mode used by `pdfium-render` when creating new [PdfPagePathObject]
    /// instances. The fill mode can be changed on an object-by-object basis by calling the
    /// [PdfPagePathObject::set_fill_and_stroke_mode()] function.
    ///
    /// More information, including visual examples, can be found in Section 4.4.2 of
    /// the PDF Reference Manual, version 1.7, on page 232.
    Winding = FPDF_FILLMODE_WINDING as isize,
}

impl PdfPathFillMode {
    #[inline]
    pub(crate) fn from_pdfium(value: c_int) -> Result<PdfPathFillMode, PdfiumError> {
        match value as u32 {
            FPDF_FILLMODE_NONE => Ok(PdfPathFillMode::None),
            FPDF_FILLMODE_ALTERNATE => Ok(PdfPathFillMode::EvenOdd),
            FPDF_FILLMODE_WINDING => Ok(PdfPathFillMode::Winding),
            _ => Err(PdfiumError::UnknownPdfPagePathFillMode),
        }
    }

    #[inline]
    #[allow(dead_code)]
    // The as_pdfium() function is not currently used, but we expect it to be in future
    pub(crate) fn as_pdfium(&self) -> c_uint {
        match self {
            PdfPathFillMode::None => FPDF_FILLMODE_NONE,
            PdfPathFillMode::EvenOdd => FPDF_FILLMODE_ALTERNATE,
            PdfPathFillMode::Winding => FPDF_FILLMODE_WINDING,
        }
    }
}

impl Default for PdfPathFillMode {
    /// Returns the default fill mode used when creating new [PdfPagePathObject]
    /// instances. The fill mode can be changed on an object-by-object basis by calling the
    /// [PdfPagePathObject::set_fill_and_stroke_mode()] function.
    #[inline]
    fn default() -> Self {
        PdfPathFillMode::Winding
    }
}

/// A single `PdfPageObject` of type `PdfPageObjectType::Path`. The page object defines a path.
///
/// Paths define shapes, trajectories, and regions of all sorts. They are used to draw
/// lines, define the shapes of filled areas, and specify boundaries for clipping other
/// graphics. A path is composed of one or more _path segments_, each specifying
/// a straight or curved line segment. Each segment may connect to one another, forming a
/// _closed sub-path_, or may be disconnected from one another, forming one or more
/// _open sub-paths_. A path therefore is made up of one or more disconnected sub-paths, each
/// comprising a sequence of connected segments. Closed sub-paths can be filled;
/// both closed and open sub-paths can be stroked. The topology of the path is unrestricted;
/// it may be concave or convex, may contain multiple sub-paths representing disjoint areas,
/// and may intersect itself in arbitrary ways.
///
/// Page objects can be created either attached to a `PdfPage` (in which case the page object's
/// memory is owned by the containing page) or detached from any page (in which case the page
/// object's memory is owned by the object). Page objects are not rendered until they are
/// attached to a page; page objects that are never attached to a page will be lost when they
/// fall out of scope.
///
/// The simplest way to create a path object that is immediately attached to a page is to call
/// one of the `PdfPageObjects::create_path_object_*()` functions to create lines, rectangles,
/// circles, and ellipses. Alternatively you can create a detached path object using one of the
/// following functions, but you must add the object to a containing `PdfPageObjects` collection manually.
///
/// * [PdfPagePathObject::new()]: creates an empty detached path object. Segments can be added to the
/// path by sequentially calling one or more of the [PdfPagePathObject::move_to()],
/// [PdfPagePathObject::line_to()], or [PdfPagePathObject::bezier_to()] functions.
/// A closed sub-path can be created by calling the [PdfPagePathObject::close_path()]
/// function. Convenience functions for adding rectangles, circles, and ellipses are also
/// available with the [PdfPagePathObject::rect_to()], [PdfPagePathObject::circle_to()],
/// and [PdfPagePathObject::ellipse_to()] functions, which create the desired shapes by
/// constructing closed sub-paths from other path segments.
/// * [PdfPagePathObject::new_line()]: creates a detached path object initialized with a single straight line.
/// * [PdfPagePathObject::new_rect()]: creates a detached path object initialized with a rectangular path.
/// * [PdfPagePathObject::new_circle()]: creates a detached path object initialized with a circular path,
/// filling the given rectangle.
/// * [PdfPagePathObject::new_circle_at()]: creates a detached path object initialized with a circular path,
/// centered at a particular origin point with a given radius.
/// * [PdfPagePathObject::new_ellipse()]: creates a detached path object initialized with an elliptical path,
/// filling the given rectangle.
/// * [PdfPagePathObject::new_ellipse_at()]: creates a detached path object initialized with an elliptical path,
/// centered at a particular origin point with given horizontal and vertical radii.
///
/// The detached path object can later be attached to a page by using the
/// `PdfPageObjects::add_path_object()` function.
pub struct PdfPagePathObject<'a> {
    object_handle: FPDF_PAGEOBJECT,
    page_handle: Option<FPDF_PAGE>,
    bindings: &'a dyn PdfiumLibraryBindings,
    current_point_x: PdfPoints,
    current_point_y: PdfPoints,
}

impl<'a> PdfPagePathObject<'a> {
    #[inline]
    pub(crate) fn from_pdfium(
        object_handle: FPDF_PAGEOBJECT,
        page_handle: FPDF_PAGE,
        bindings: &'a dyn PdfiumLibraryBindings,
    ) -> Self {
        PdfPagePathObject {
            object_handle,
            page_handle: Some(page_handle),
            bindings,
            current_point_x: PdfPoints::ZERO,
            current_point_y: PdfPoints::ZERO,
        }
    }

    /// Creates a new [PdfPagePathObject] from the given arguments. The returned page object
    /// will not be rendered until it is added to a `PdfPage` using the
    /// `PdfPageObjects::add_path_object()` function.
    ///
    /// The new path will be created with the given initial position and with the given fill and stroke
    /// settings applied. Both the stroke color and the stroke width must be provided for the
    /// path to be stroked.
    ///
    /// Other than setting the initial position, this path will be empty. Add additional segments
    /// to this path by calling one or more of the [PdfPagePathObject::move_to()],
    /// [PdfPagePathObject::line_to()], or [PdfPagePathObject::bezier_to()]
    /// functions. A closed sub-path can be created by calling the [PdfPagePathObject::close_path()]
    /// function. Convenience functions for adding rectangles, circles, and ellipses are also
    /// available with the [PdfPagePathObject::rect_to()], [PdfPagePathObject::circle_to()],
    /// and [PdfPagePathObject::ellipse_to()] functions, which create the desired shapes by
    /// constructing closed sub-paths from other path segments.
    #[inline]
    pub fn new(
        document: &'a PdfDocument<'a>,
        x: PdfPoints,
        y: PdfPoints,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        Self::new_from_bindings(
            document.get_bindings(),
            x,
            y,
            stroke_color,
            stroke_width,
            fill_color,
        )
    }

    pub(crate) fn new_from_bindings(
        bindings: &'a dyn PdfiumLibraryBindings,
        x: PdfPoints,
        y: PdfPoints,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        let handle = bindings.FPDFPageObj_CreateNewPath(x.value, y.value);

        if handle.is_null() {
            if let Some(error) = bindings.get_pdfium_last_error() {
                Err(PdfiumError::PdfiumLibraryInternalError(error))
            } else {
                // This would be an unusual situation; a null handle indicating failure,
                // yet Pdfium's error code indicates success.

                Err(PdfiumError::PdfiumLibraryInternalError(
                    PdfiumInternalError::Unknown,
                ))
            }
        } else {
            let mut result = PdfPagePathObject {
                object_handle: handle,
                page_handle: None,
                bindings,
                current_point_x: x,
                current_point_y: y,
            };

            result.move_to(x, y)?;

            let do_stroke = if let Some(stroke_color) = stroke_color {
                if let Some(stroke_width) = stroke_width {
                    result.set_stroke_color(stroke_color)?;
                    result.set_stroke_width(stroke_width)?;

                    true
                } else {
                    false
                }
            } else {
                false
            };

            let fill_mode = if let Some(fill_color) = fill_color {
                result.set_fill_color(fill_color)?;

                PdfPathFillMode::default()
            } else {
                PdfPathFillMode::None
            };

            result.set_fill_and_stroke_mode(fill_mode, do_stroke)?;

            Ok(result)
        }
    }

    #[inline]
    pub(crate) fn new_line_from_bindings(
        bindings: &'a dyn PdfiumLibraryBindings,
        x1: PdfPoints,
        y1: PdfPoints,
        x2: PdfPoints,
        y2: PdfPoints,
        stroke_color: PdfColor,
        stroke_width: PdfPoints,
    ) -> Result<Self, PdfiumError> {
        let mut result = Self::new_from_bindings(
            bindings,
            x1,
            y1,
            Some(stroke_color),
            Some(stroke_width),
            None,
        )?;

        result.line_to(x2, y2)?;

        Ok(result)
    }

    /// Creates a new [PdfPagePathObject] from the given arguments. The returned page object
    /// will not be rendered until it is added to a `PdfPage` using the
    /// `PdfPageObjects::add_path_object()` function.
    ///
    /// The new path will be created with a line with the given start and end coordinates,
    /// with the given stroke settings applied.
    #[inline]
    pub fn new_line(
        document: &'a PdfDocument<'a>,
        x1: PdfPoints,
        y1: PdfPoints,
        x2: PdfPoints,
        y2: PdfPoints,
        stroke_color: PdfColor,
        stroke_width: PdfPoints,
    ) -> Result<Self, PdfiumError> {
        Self::new_line_from_bindings(
            document.get_bindings(),
            x1,
            y1,
            x2,
            y2,
            stroke_color,
            stroke_width,
        )
    }

    #[inline]
    pub(crate) fn new_rect_from_bindings(
        bindings: &'a dyn PdfiumLibraryBindings,
        rect: PdfRect,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        let mut result = Self::new_from_bindings(
            bindings,
            rect.left,
            rect.bottom,
            stroke_color,
            stroke_width,
            fill_color,
        )?;

        result.rect_to(rect.right, rect.top)?;

        Ok(result)
    }

    /// Creates a new [PdfPagePathObject] from the given arguments. The returned page object
    /// will not be rendered until it is added to a `PdfPage` using the
    /// `PdfPageObjects::add_path_object()` function.
    ///
    /// The new path will be created with a path for the given rectangle, with the given
    /// fill and stroke settings applied. Both the stroke color and the stroke width must be
    /// provided for the rectangle to be stroked.
    #[inline]
    pub fn new_rect(
        document: &'a PdfDocument<'a>,
        rect: PdfRect,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        Self::new_rect_from_bindings(
            document.get_bindings(),
            rect,
            stroke_color,
            stroke_width,
            fill_color,
        )
    }

    #[inline]
    pub(crate) fn new_circle_from_bindings(
        bindings: &'a dyn PdfiumLibraryBindings,
        rect: PdfRect,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        let mut result = Self::new_from_bindings(
            bindings,
            rect.left,
            rect.bottom,
            stroke_color,
            stroke_width,
            fill_color,
        )?;

        result.circle_to(rect.right, rect.top)?;

        Ok(result)
    }

    /// Creates a new [PdfPagePathObject] from the given arguments. The returned page object
    /// will not be rendered until it is added to a `PdfPage` using the
    /// `PdfPageObjects::add_path_object()` function.
    ///
    /// The new path will be created with a circle that fills the given rectangle, with the given
    /// fill and stroke settings applied. Both the stroke color and the stroke width must be
    /// provided for the circle to be stroked.
    #[inline]
    pub fn new_circle(
        document: &'a PdfDocument<'a>,
        rect: PdfRect,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        Self::new_circle_from_bindings(
            document.get_bindings(),
            rect,
            stroke_color,
            stroke_width,
            fill_color,
        )
    }

    #[inline]
    pub(crate) fn new_circle_at_from_bindings(
        bindings: &'a dyn PdfiumLibraryBindings,
        center_x: PdfPoints,
        center_y: PdfPoints,
        radius: PdfPoints,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        Self::new_circle_from_bindings(
            bindings,
            PdfRect::new(
                center_y - radius,
                center_x - radius,
                center_y + radius,
                center_x + radius,
            ),
            stroke_color,
            stroke_width,
            fill_color,
        )
    }

    /// Creates a new [PdfPagePathObject] from the given arguments. The returned page object
    /// will not be rendered until it is added to a `PdfPage` using the
    /// `PdfPageObjects::add_path_object()` function.
    ///
    /// The new path will be created with a circle centered at the given coordinates, with the
    /// given radius, and with the given fill and stroke settings applied. Both the stroke color
    /// and the stroke width must be provided for the circle to be stroked.
    #[inline]
    pub fn new_circle_at(
        document: &'a PdfDocument<'a>,
        center_x: PdfPoints,
        center_y: PdfPoints,
        radius: PdfPoints,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        Self::new_circle_at_from_bindings(
            document.get_bindings(),
            center_x,
            center_y,
            radius,
            stroke_color,
            stroke_width,
            fill_color,
        )
    }

    #[inline]
    pub(crate) fn new_ellipse_from_bindings(
        bindings: &'a dyn PdfiumLibraryBindings,
        rect: PdfRect,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        let mut result = Self::new_from_bindings(
            bindings,
            rect.left,
            rect.bottom,
            stroke_color,
            stroke_width,
            fill_color,
        )?;

        result.ellipse_to(rect.right, rect.top)?;

        Ok(result)
    }

    /// Creates a new [PdfPagePathObject] from the given arguments. The returned page object
    /// will not be rendered until it is added to a `PdfPage` using the
    /// `PdfPageObjects::add_path_object()` function.
    ///
    /// The new path will be created with an ellipse that fills the given rectangle, with the given
    /// fill and stroke settings applied. Both the stroke color and the stroke width must be
    /// provided for the ellipse to be stroked.
    #[inline]
    pub fn new_ellipse(
        document: &'a PdfDocument<'a>,
        rect: PdfRect,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        Self::new_ellipse_from_bindings(
            document.get_bindings(),
            rect,
            stroke_color,
            stroke_width,
            fill_color,
        )
    }

    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub(crate) fn new_ellipse_at_from_bindings(
        bindings: &'a dyn PdfiumLibraryBindings,
        center_x: PdfPoints,
        center_y: PdfPoints,
        x_radius: PdfPoints,
        y_radius: PdfPoints,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        Self::new_ellipse_from_bindings(
            bindings,
            PdfRect::new(
                center_y - y_radius,
                center_x - x_radius,
                center_y + y_radius,
                center_x + x_radius,
            ),
            stroke_color,
            stroke_width,
            fill_color,
        )
    }

    /// Creates a new [PdfPagePathObject] from the given arguments. The returned page object
    /// will not be rendered until it is added to a `PdfPage` using the
    /// `PdfPageObjects::add_path_object()` function.
    ///
    /// The new path will be created with an ellipse centered at the given coordinates, with the
    /// given horizontal and vertical radii, and with the given fill and stroke settings applied.
    /// Both the stroke color and the stroke width must be provided for the ellipse to be stroked.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn new_ellipse_at(
        document: &'a PdfDocument<'a>,
        center_x: PdfPoints,
        center_y: PdfPoints,
        x_radius: PdfPoints,
        y_radius: PdfPoints,
        stroke_color: Option<PdfColor>,
        stroke_width: Option<PdfPoints>,
        fill_color: Option<PdfColor>,
    ) -> Result<Self, PdfiumError> {
        Self::new_ellipse_at_from_bindings(
            document.get_bindings(),
            center_x,
            center_y,
            x_radius,
            y_radius,
            stroke_color,
            stroke_width,
            fill_color,
        )
    }

    /// Begins a new sub-path in this [PdfPagePathObject] by moving the current point to the
    /// given coordinates, omitting any connecting line segment.
    pub fn move_to(&mut self, x: PdfPoints, y: PdfPoints) -> Result<(), PdfiumError> {
        if self.bindings.is_true(self.bindings.FPDFPath_MoveTo(
            self.object_handle,
            x.value,
            y.value,
        )) {
            self.current_point_x = x;
            self.current_point_y = y;

            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                self.bindings
                    .get_pdfium_last_error()
                    .unwrap_or(PdfiumInternalError::Unknown),
            ))
        }
    }

    /// Appends a straight line segment to this [PdfPagePathObject] from the current point to the
    /// given coordinates. The new current point is set to the given coordinates.
    pub fn line_to(&mut self, x: PdfPoints, y: PdfPoints) -> Result<(), PdfiumError> {
        if self.bindings.is_true(self.bindings.FPDFPath_LineTo(
            self.object_handle,
            x.value,
            y.value,
        )) {
            self.current_point_x = x;
            self.current_point_y = y;

            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                self.bindings
                    .get_pdfium_last_error()
                    .unwrap_or(PdfiumInternalError::Unknown),
            ))
        }
    }

    /// Appends a cubic Bézier curve to this [PdfPagePathObject] from the current point to the
    /// given coordinates, using the two given Bézier control points. The new current point
    /// is set to the given coordinates.
    pub fn bezier_to(
        &mut self,
        x: PdfPoints,
        y: PdfPoints,
        control1_x: PdfPoints,
        control1_y: PdfPoints,
        control2_x: PdfPoints,
        control2_y: PdfPoints,
    ) -> Result<(), PdfiumError> {
        if self.bindings.is_true(self.bindings.FPDFPath_BezierTo(
            self.object_handle,
            control1_x.value,
            control1_y.value,
            control2_x.value,
            control2_y.value,
            x.value,
            y.value,
        )) {
            self.current_point_x = x;
            self.current_point_y = y;

            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                self.bindings
                    .get_pdfium_last_error()
                    .unwrap_or(PdfiumInternalError::Unknown),
            ))
        }
    }

    /// Appends a rectangle to this [PdfPagePathObject] by drawing four line segments
    /// from the current point, ending at the given coordinates. The current sub-path will be closed.
    /// The new current point is set to the given coordinates.
    pub fn rect_to(&mut self, x: PdfPoints, y: PdfPoints) -> Result<(), PdfiumError> {
        let orig_x = self.current_point_x;

        let orig_y = self.current_point_y;

        self.close_path()?;
        self.line_to(orig_x, y)?;
        self.line_to(x, y)?;
        self.line_to(x, orig_y)?;
        self.close_path()?;
        self.move_to(x, y)
    }

    /// Appends an ellipse to this [PdfPagePathObject] by drawing four Bézier curves approximating
    /// an ellipse filling a rectangle from the current point to the given coordinates.
    /// The current sub-path will be closed. The new current point is set to the given coordinates.
    pub fn ellipse_to(&mut self, x: PdfPoints, y: PdfPoints) -> Result<(), PdfiumError> {
        let x_radius = (x - self.current_point_x) / 2.0;

        let y_radius = (y - self.current_point_y) / 2.0;

        self.close_path()?;
        self.move_to(
            self.current_point_x + x_radius,
            self.current_point_y + y_radius,
        )?;
        self.ellipse(x_radius, y_radius)?;
        self.move_to(x, y)
    }

    /// Appends a circle to this [PdfPagePathObject] by drawing four Bézier curves approximating
    /// a circle filling a rectangle from the current point to the given coordinates.
    /// The current sub-path will be closed. The new current point is set to the given coordinates.
    ///
    /// Note that perfect circles cannot be represented exactly using Bézier curves. However,
    /// a very close approximation, more than sufficient to please the human eye, can be achieved
    /// using four Bézier curves, one for each quadrant of the circle.
    pub fn circle_to(&mut self, x: PdfPoints, y: PdfPoints) -> Result<(), PdfiumError> {
        let radius = (x - self.current_point_x) / 2.0;

        self.move_to(self.current_point_x + radius, self.current_point_y + radius)?;
        self.ellipse(radius, radius)?;
        self.move_to(x, y)
    }

    /// Draws an ellipse at the current point using the given horizontal and vertical radii.
    /// The ellipse will be constructed using four Bézier curves, one for each quadrant.
    fn ellipse(&mut self, x_radius: PdfPoints, y_radius: PdfPoints) -> Result<(), PdfiumError> {
        // Ellipse approximation method: https://spencermortensen.com/articles/bezier-circle/
        // Implementation based on: https://stackoverflow.com/a/2007782

        const C: f32 = 0.551915;

        let x_c = x_radius * C;

        let y_c = y_radius * C;

        let orig_x = self.current_point_x;

        let orig_y = self.current_point_y;

        self.move_to(orig_x - x_radius, orig_y)?;
        self.bezier_to(
            orig_x,
            orig_y + y_radius,
            orig_x - x_radius,
            orig_y + y_c,
            orig_x - x_c,
            orig_y + y_radius,
        )?;
        self.bezier_to(
            orig_x + x_radius,
            orig_y,
            orig_x + x_c,
            orig_y + y_radius,
            orig_x + x_radius,
            orig_y + y_c,
        )?;
        self.bezier_to(
            orig_x,
            orig_y - y_radius,
            orig_x + x_radius,
            orig_y - y_c,
            orig_x + x_c,
            orig_y - y_radius,
        )?;
        self.bezier_to(
            orig_x - x_radius,
            orig_y,
            orig_x - x_c,
            orig_y - y_radius,
            orig_x - x_radius,
            orig_y - y_c,
        )?;
        self.close_path()
    }

    /// Closes the current sub-path in this [PdfPagePathObject] by appending a straight line segment
    /// from the current point to the starting point of the sub-path.
    pub fn close_path(&mut self) -> Result<(), PdfiumError> {
        if self
            .bindings
            .is_true(self.bindings.FPDFPath_Close(self.object_handle))
        {
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                self.bindings
                    .get_pdfium_last_error()
                    .unwrap_or(PdfiumInternalError::Unknown),
            ))
        }
    }

    /// Returns the method used to determine which sub-paths of any path in this [PdfPagePathObject]
    /// should be filled.
    #[inline]
    pub fn fill_mode(&self) -> Result<PdfPathFillMode, PdfiumError> {
        let mut raw_fill_mode: c_int = 0;

        let mut _raw_stroke: FPDF_BOOL = self.get_bindings().FALSE();

        if self
            .get_bindings()
            .is_true(self.get_bindings().FPDFPath_GetDrawMode(
                *self.get_object_handle(),
                &mut raw_fill_mode,
                &mut _raw_stroke,
            ))
        {
            PdfPathFillMode::from_pdfium(raw_fill_mode)
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                self.get_bindings()
                    .get_pdfium_last_error()
                    .unwrap_or(PdfiumInternalError::Unknown),
            ))
        }
    }

    /// Returns `true` if this [PdfPagePathObject] will be stroked, regardless of the path's
    /// stroke settings.
    ///
    /// Even if this path is set to be stroked, the stroke must be configured with a visible color
    /// and a non-zero width in order to actually be visible.
    #[inline]
    pub fn is_stroked(&self) -> Result<bool, PdfiumError> {
        let mut _raw_fill_mode: c_int = 0;

        let mut raw_stroke: FPDF_BOOL = self.get_bindings().FALSE();

        if self
            .get_bindings()
            .is_true(self.get_bindings().FPDFPath_GetDrawMode(
                *self.get_object_handle(),
                &mut _raw_fill_mode,
                &mut raw_stroke,
            ))
        {
            Ok(self.get_bindings().is_true(raw_stroke))
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                self.get_bindings()
                    .get_pdfium_last_error()
                    .unwrap_or(PdfiumInternalError::Unknown),
            ))
        }
    }

    /// Sets the method used to determine which sub-paths of any path in this [PdfPagePathObject]
    /// should be filled, and whether or not any path in this [PdfPagePathObject] should be stroked.
    ///
    /// Even if this object's path is set to be stroked, the stroke must be configured with
    /// a visible color and a non-zero width in order to actually be visible.
    #[inline]
    pub fn set_fill_and_stroke_mode(
        &mut self,
        fill_mode: PdfPathFillMode,
        do_stroke: bool,
    ) -> Result<(), PdfiumError> {
        if self
            .get_bindings()
            .is_true(self.get_bindings().FPDFPath_SetDrawMode(
                *self.get_object_handle(),
                fill_mode.as_pdfium() as c_int,
                self.get_bindings().bool_to_pdfium(do_stroke),
            ))
        {
            Ok(())
        } else {
            Err(PdfiumError::PdfiumLibraryInternalError(
                self.get_bindings()
                    .get_pdfium_last_error()
                    .unwrap_or(PdfiumInternalError::Unknown),
            ))
        }
    }
}

impl<'a> PdfPageObjectPrivate<'a> for PdfPagePathObject<'a> {
    #[inline]
    fn get_object_handle(&self) -> &FPDF_PAGEOBJECT {
        &self.object_handle
    }

    #[inline]
    fn get_page_handle(&self) -> &Option<FPDF_PAGE> {
        &self.page_handle
    }

    #[inline]
    fn set_page_handle(&mut self, page: FPDF_PAGE) {
        self.page_handle = Some(page);
    }

    #[inline]
    fn clear_page_handle(&mut self) {
        self.page_handle = None;
    }

    #[inline]
    fn get_bindings(&self) -> &dyn PdfiumLibraryBindings {
        self.bindings
    }
}
