//
//  DetailView.swift
//  Picosso
//
//  Created by Ritam Sarmah on 3/23/23.
//

import SwiftUI

struct DetailView: View {
    
    @Environment(\.dismiss) var dismiss
    
    @ObservedObject var viewModel: DetailViewModel
    
    var body: some View {
        NavigationView {
            VStack {
                if let uiImage = viewModel.uiImage {
                    Image(uiImage: uiImage)
                        .resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(maxHeight: 400)
                }
                
                Form {
                    Section {
                        TextField("Title", text: $viewModel.title)
                        TextField("Artist", text: $viewModel.artist)
                        Toggle("Allow White Padding", isOn: $viewModel.shouldPad)
                    }
                    
                    Section {
                        Toggle("Replace Existing Art", isOn: $viewModel.shouldOverwrite)
                    }
                }
            }
            .navigationTitle("Add Artwork")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel", role: .cancel) {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .navigationBarTrailing) {
                    Button("Save") {
                        Task {
                            if await viewModel.save() {
                                dismiss()
                            }
                        }
                    }
                    .disabled(viewModel.title.isEmpty)
                }
            }
        }
        .errorAlert(info: viewModel.errorInfo)
    }
}

//struct UploadView_Previews: PreviewProvider {
//    static var previews: some View {
//        DetailView()
//    }
//}
