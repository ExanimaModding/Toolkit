# TODO: export emtk module

# TODO: no module named 'emtk'
# import emtk
import bpy
from bpy.props import StringProperty, CollectionProperty
from bpy.types import Operator
from bpy_extras.io_utils import ImportHelper, ExportHelper


class ExportRfc(Operator, ExportHelper):
    """Export scene as Rfc file"""

    bl_idname = "export_scene.rfc"
    bl_label = "Export Rfc"
    filename_ext = ".rfc"
    filter_glob: StringProperty(default="*.rfc", options={"HIDDEN"})  # pyright: ignore[reportInvalidTypeForm]

    @staticmethod
    def register():
        bpy.types.TOPBAR_MT_file_export.append(ExportRfc.__draw_class)

    @staticmethod
    def unregister():
        bpy.types.TOPBAR_MT_file_export.remove(ExportRfc.__draw_class)

    @staticmethod
    def __draw_class(bl_type, context):
        bl_type.layout.operator(ExportRfc.bl_idname, text="Rayform Content (.rfc)")


class ImportRfc(Operator, ImportHelper):
    """Load an Rfc file"""

    bl_idname = "import_scene.rfc"
    bl_label = "Import Rfc"
    bl_options = {"REGISTER", "UNDO"}

    filter_glob: StringProperty(default="*.rfc", options={"HIDDEN"})  # pyright: ignore[reportInvalidTypeForm]

    files: CollectionProperty(name="File Path", type=bpy.types.OperatorFileListElement)  # pyright: ignore[reportInvalidTypeForm]

    def execute(self, context):
        self.report({"INFO"}, "Hello rayform content")

        # TODO: use exparser here
        if self.files:
            for file in self.files:
                self.report({"INFO"}, "files picked")
        else:
            self.report({"INFO"}, "canceled")

        return {"FINISHED"}

    @staticmethod
    def register():
        bpy.types.TOPBAR_MT_file_import.append(ImportRfc.__draw_class)

    @staticmethod
    def unregister():
        bpy.types.TOPBAR_MT_file_import.remove(ImportRfc.__draw_class)

    @staticmethod
    def __draw_class(bl_type, context):
        bl_type.layout.operator(ImportRfc.bl_idname, text="Rayform Content (.rfc)")


classes = (ExportRfc, ImportRfc)


def register():
    for cls in classes:
        bpy.utils.register_class(cls)


def unregister():
    for cls in classes:
        bpy.utils.unregister_class(cls)


# NOTE: development
if __name__ == "__main__":
    register()
